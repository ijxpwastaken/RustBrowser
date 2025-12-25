//! Debug test for DOM tree

use html_parser::parse;

fn main() {
    let html = r#"<html><body><p>Hello World</p></body></html>"#;
    
    println!("Parsing HTML: {}", html);
    
    match parse(html) {
        Ok(doc) => {
            println!("Parse successful!");
            println!("Doctype: {:?}", doc.doctype);
            println!("Document element: {:?}", doc.document_element.is_some());
            
            if let Some(ref root) = doc.document_element {
                if let Ok(node) = root.read() {
                    println!("Root node type: {:?}", node.node_type());
                    if let Some(elem) = node.as_element() {
                        println!("Root tag: {}", elem.tag_name);
                        println!("Root children count: {}", elem.children.len());
                        
                        // Traverse children
                        for (i, child) in elem.children.iter().enumerate() {
                            if let Ok(child_node) = child.read() {
                                println!("  Child {}: {:?}", i, child_node.node_type());
                                if let Some(child_elem) = child_node.as_element() {
                                    println!("    Tag: {}", child_elem.tag_name);
                                    println!("    Children count: {}", child_elem.children.len());
                                    
                                    for (j, grandchild) in child_elem.children.iter().enumerate() {
                                        if let Ok(gc_node) = grandchild.read() {
                                            println!("      Grandchild {}: {:?}", j, gc_node.node_type());
                                            if let Some(gc_elem) = gc_node.as_element() {
                                                println!("        Tag: {}", gc_elem.tag_name);
                                                println!("        Children: {}", gc_elem.children.len());
                                                
                                                for (k, ggc) in gc_elem.children.iter().enumerate() {
                                                    if let Ok(ggc_node) = ggc.read() {
                                                        println!("          GGC {}: {:?}", k, ggc_node.node_type());
                                                        if let Some(text) = ggc_node.as_text() {
                                                            println!("            Text: '{}'", text.content);
                                                        }
                                                    }
                                                }
                                            }
                                            if let Some(text) = gc_node.as_text() {
                                                println!("        Text: '{}'", text.content);
                                            }
                                        }
                                    }
                                }
                                if let Some(text) = child_node.as_text() {
                                    println!("    Text: '{}'", text.content);
                                }
                            }
                        }
                    }
                }
            }
            
            // Try the body() method
            if let Some(body) = doc.body() {
                println!("\nFound body element!");
                if let Ok(body_node) = body.read() {
                    if let Some(body_elem) = body_node.as_element() {
                        println!("Body children: {}", body_elem.children.len());
                    }
                }
            } else {
                println!("\nBody NOT found!");
            }
            
            println!("\nPrinting tree:");
            doc.print_tree();
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
}
