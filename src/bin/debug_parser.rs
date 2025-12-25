//! Debug test for parser

use html_parser::parse;

fn main() {
    let html = r##"<!doctype html>
<html lang="en">
  <head>
    <title>HTML5 Test Page</title>
  </head>
  <body>
    <header>
      <h1>HTML5 Test Page</h1>
      <p>This is a test page.</p>
    </header>
  </body>
</html>"##;

    println!("=== Parsing complex HTML ===");
    println!("HTML length: {} bytes", html.len());
    
    match parse(html) {
        Ok(doc) => {
            println!("Parse OK!");
            println!("Doctype: {:?}", doc.doctype);
            println!("Has document_element: {}", doc.document_element.is_some());
            
            if let Some(ref root) = doc.document_element {
                if let Ok(node) = root.read() {
                    println!("Root node: {:?}", node.node_type());
                    if let Some(elem) = node.as_element() {
                        println!("Root tag: <{}>", elem.tag_name);
                        println!("Root children: {}", elem.children.len());
                    }
                }
            } else {
                println!("ERROR: No document_element!");
            }
            
            println!("\n=== Tree Print ===");
            doc.print_tree();
        }
        Err(e) => {
            println!("Parse ERROR: {:?}", e);
        }
    }
}
