use xmltree::Element;

/// Recursivley looks for a image element in an xml file
///
/// # Arguments
///
/// * `element` - The parent element to look for image siblings in
pub fn find_img_element(element: &Element) -> Option<&Element> {
    if element.name == "img" {
        Some(element)
    } else {
        for child in &element.children {
            if let Some(child_element) = child.as_element() {
                if let Some(img_element) = find_img_element(child_element) {
                    return Some(img_element);
                }
            }
        }
        None
    }
}

/// Extracts the file name from a image element
///
/// # Arguments
///
/// * `element` - The image element to get the src attribute from
pub fn extract_image_source(element: &Element) -> Option<String> {
    if let Some(source_element) = find_img_element(element) {
        if let Some(src) = source_element.attributes.get("src") {
            if let (Some(last_slash), Some(last_dot)) = (src.rfind('/'), src.rfind('.')) {
                let filename = &src[last_slash + 1..last_dot];
                return Some(filename.to_string());
            }
        }
    }

    None
}
