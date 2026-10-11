use crate::AssetError;
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

/// 内部格式树；拒绝未知语法，不为截断文本补闭合标签。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    pub tag: String,
    pub text: String,
    pub children: Vec<Node>,
}

pub(crate) fn error(message: impl Into<String>) -> AssetError {
    AssetError::Xml {
        message: message.into(),
    }
}

fn node(start: &BytesStart<'_>) -> Result<Node, AssetError> {
    if start.attributes().next().is_some() {
        return Err(error("当前格式不支持 XML 属性"));
    }
    let tag = std::str::from_utf8(start.name().as_ref())
        .map_err(|_| error("标签不是 UTF-8"))?
        .to_owned();
    Ok(Node {
        tag,
        text: String::new(),
        children: Vec::new(),
    })
}
fn append(stack: &mut [Node], roots: &mut Vec<Node>, mut node: Node) {
    // 只去掉标量外侧空白；曲线内部标点和空白逐字保留。
    if node.tag != "text" {
        node.text = node.text.trim().to_owned();
    }
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}
fn text(stack: &mut [Node], value: &str) -> Result<(), AssetError> {
    if !value.chars().all(xml_character) {
        return Err(error("文本包含 XML 1.0 不允许的字符"));
    }
    if let Some(parent) = stack.last_mut() {
        parent.text.push_str(value);
        Ok(())
    } else if value.trim().is_empty() {
        Ok(())
    } else {
        Err(error("顶层存在非空文本"))
    }
}
pub(crate) fn parse(source: &str) -> Result<Vec<Node>, AssetError> {
    if !source.chars().all(xml_character) {
        return Err(error("输入包含 XML 1.0 不允许的字符"));
    }
    let mut reader = Reader::from_str(source);
    reader.config_mut().check_comments = true;
    let mut stack = Vec::new();
    let mut roots = Vec::new();
    loop {
        match reader.read_event().map_err(|_| {
            error(format!(
                "XML 语法错误（字节位置 {}）",
                reader.error_position()
            ))
        })? {
            Event::Start(start) => {
                if stack.len() >= 64 {
                    return Err(error("XML 嵌套深度超出格式限制"));
                }
                stack.push(node(&start)?);
            }
            Event::Empty(empty) => append(&mut stack, &mut roots, node(&empty)?),
            Event::End(end) => {
                let finished = stack
                    .pop()
                    .ok_or_else(|| error("闭合标签没有对应开始标签"))?;
                if finished.tag.as_bytes() != end.name().as_ref() {
                    return Err(error("标签闭合顺序不一致"));
                }
                append(&mut stack, &mut roots, finished);
            }
            Event::Text(value) => {
                if value.as_ref().windows(3).any(|part| part == b"]]>") {
                    return Err(error("普通文本不能包含 CDATA 闭合分隔符"));
                }
                text(
                    &mut stack,
                    &value.decode().map_err(|_| error("文本解码失败"))?,
                )?;
            }
            Event::CData(value) => text(
                &mut stack,
                &value.decode().map_err(|_| error("CDATA 解码失败"))?,
            )?,
            Event::GeneralRef(value) => {
                let name = value.decode().map_err(|_| error("实体引用解码失败"))?;
                let encoded = format!("&{name};");
                let decoded = quick_xml::escape::unescape(&encoded)
                    .map_err(|_| error("未知或无效的 XML 实体引用"))?;
                text(&mut stack, &decoded)?;
            }
            Event::Comment(_) => {}
            Event::Eof => break,
            _ => return Err(error("不支持 XML 声明、DTD 或处理指令")),
        }
    }
    if !stack.is_empty() {
        return Err(error("XML 文本被截断，仍有未闭合标签"));
    }
    if roots.is_empty() {
        return Err(error("XML 文本为空"));
    }
    Ok(roots)
}

fn xml_character(value: char) -> bool {
    matches!(value, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
}

pub(crate) fn container(node: &Node) -> Result<(), AssetError> {
    if !node.text.is_empty() {
        return Err(error(format!("容器 {} 中存在非空文本", node.tag)));
    }
    Ok(())
}
pub(crate) fn scalar(node: &Node) -> Result<&str, AssetError> {
    if !node.children.is_empty() {
        return Err(error(format!("标量 {} 中不能嵌套标签", node.tag)));
    }
    Ok(&node.text)
}
pub(crate) fn finite(value: &str, tag: &str) -> Result<f32, AssetError> {
    let number = value
        .parse::<f32>()
        .map_err(|_| error(format!("{tag} 必须是数值")))?;
    if !number.is_finite() {
        return Err(error(format!("{tag} 必须是有限数")));
    }
    Ok(number)
}
