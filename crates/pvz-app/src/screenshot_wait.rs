use std::time::Duration;

const WAIT_LIMIT: Duration = Duration::from_secs(30);
const TIMEOUT: &str = "GPU 截图等待超过最终上限（30 秒），未取得可用回调";

/// 截图请求使用真实时钟；成功、保存失败及缺失回调都只交付一次退出结果。
pub(super) struct ScreenshotWait {
    deadline: Duration,
    completion: Option<Result<(), String>>,
    finished: bool,
}

impl ScreenshotWait {
    pub(super) fn new(now: Duration) -> Self {
        Self {
            deadline: now.saturating_add(WAIT_LIMIT),
            completion: None,
            finished: false,
        }
    }

    pub(super) fn complete(&mut self, now: Duration, result: Result<(), String>) {
        if !self.finished && self.completion.is_none() {
            self.completion = Some(if now >= self.deadline {
                Err(TIMEOUT.to_owned())
            } else {
                result
            });
        }
    }

    pub(super) fn poll(&mut self, now: Duration) -> Option<Result<(), String>> {
        if self.finished {
            return None;
        }
        let result = self
            .completion
            .take()
            .or_else(|| (now >= self.deadline).then(|| Err(TIMEOUT.to_owned())));
        if result.is_some() {
            self.finished = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 无回调在真实时间边界失败且不重复退出() {
        let mut wait = ScreenshotWait::new(Duration::from_secs(10));
        assert!(wait.poll(Duration::from_millis(39999)).is_none());
        assert!(wait.poll(Duration::from_secs(40)).unwrap().is_err());
        assert!(wait.poll(Duration::from_secs(100)).is_none());
        wait.complete(Duration::from_secs(100), Ok(()));
        assert!(wait.poll(Duration::from_secs(100)).is_none());
    }

    #[test]
    fn 上限前成功回调只交付一次成功() {
        let mut wait = ScreenshotWait::new(Duration::ZERO);
        wait.complete(Duration::from_millis(29999), Ok(()));
        assert_eq!(wait.poll(Duration::from_secs(30)), Some(Ok(())));
        assert_eq!(wait.poll(Duration::from_secs(31)), None);
    }

    #[test]
    fn 保存失败不会被后续成功回调覆盖() {
        let mut wait = ScreenshotWait::new(Duration::ZERO);
        wait.complete(Duration::from_secs(1), Err("保存失败".to_owned()));
        wait.complete(Duration::from_secs(2), Ok(()));
        assert_eq!(
            wait.poll(Duration::from_secs(2)),
            Some(Err("保存失败".to_owned()))
        );
    }

    #[test]
    fn 上限处到达的成功回调仍然失败() {
        let mut wait = ScreenshotWait::new(Duration::ZERO);
        wait.complete(Duration::from_secs(30), Ok(()));
        assert!(wait.poll(Duration::from_secs(30)).unwrap().is_err());
    }
}
