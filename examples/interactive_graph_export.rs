//! Interactive Graph Export Demo
//!
//! This demo creates an interactive HTML visualization of person relationships
//! that can be viewed in a web browser with zoom, pan, and hover details.

use cim_domain_person::{
    aggregate::{Person, PersonId},
    services::PersonCompositionService,
    value_objects::{
        SocialMediaComponent, SegmentationComponent, CustomerSegment,
        BehavioralComponent, SkillsComponent,
    },
};

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== Interactive Graph Export Demo ===\n");

    // Create a simple network
    let service = PersonCompositionService::new();
    let mut people = HashMap::new();
    let mut relationships = Vec::new();

    // Create some people
    let ceo_id = PersonId::new();
    let ceo = service.create_employee(ceo_id, "Victoria Sterling", "Executive", Some("CEO"), None);
    people.insert(ceo_id, ("Victoria Sterling", "CEO", "executive"));

    let cto_id = PersonId::new();
    let cto = service.create_employee(cto_id, "Marcus Chen", "Technology", Some("CTO"), Some(ceo_id));
    people.insert(cto_id, ("Marcus Chen", "CTO", "executive"));
    relationships.push((ceo_id, cto_id, "manages"));

    let dev_id = PersonId::new();
    let dev = service.create_employee(dev_id, "Alice Wong", "Engineering", Some("Lead Developer"), Some(cto_id));
    people.insert(dev_id, ("Alice Wong", "Lead Developer", "employee"));
    relationships.push((cto_id, dev_id, "manages"));

    let customer_id = PersonId::new();
    let customer = service.create_customer(customer_id, "TechCorp", Some("info@techcorp.com"), Some("+1-555-0100"));
    people.insert(customer_id, ("TechCorp", "VIP Customer", "customer"));
    relationships.push((ceo_id, customer_id, "business_relationship"));

    // Generate interactive HTML
    let html_content = generate_interactive_html(&people, &relationships);

    // Write to file
    let filename = "person_relationship_graph.html";
    match File::create(filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(html_content.as_bytes()) {
                eprintln!("Failed to write HTML file: {}", e);
            } else {
                println!("✅ Interactive graph exported to: {}", filename);
                println!("\nTo view the graph:");
                println!("  1. Open {} in a web browser", filename);
                println!("  2. Use mouse to drag nodes around");
                println!("  3. Hover over nodes for details");
                println!("  4. Zoom with mouse wheel");
                println!("  5. Click nodes to highlight connections");
            }
        }
        Err(e) => eprintln!("Failed to create HTML file: {}", e),
    }

    // Also generate a JSON export for other tools
    let json_content = generate_json_export(&people, &relationships);
    match File::create("person_relationship_data.json") {
        Ok(mut file) => {
            if let Err(e) = file.write_all(json_content.as_bytes()) {
                eprintln!("Failed to write JSON file: {}", e);
            } else {
                println!("\n📊 Data also exported to: person_relationship_data.json");
            }
        }
        Err(e) => eprintln!("Failed to create JSON file: {}", e),
    }
}

fn generate_interactive_html(
    people: &HashMap<PersonId, (&str, &str, &str)>,
    relationships: &Vec<(PersonId, PersonId, &str)>,
) -> String {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut id_map = HashMap::new();

    // Create nodes
    for (i, (person_id, (name, role, category))) in people.iter().enumerate() {
        id_map.insert(person_id, i);
        
        let color = match *category {
            "executive" => "#ff6b6b",
            "employee" => "#4ecdc4",
            "customer" => "#45b7d1",
            _ => "#dfe6e9",
        };

        nodes.push(format!(
            r#"{{
                id: {},
                label: '{}',
                title: '{}: {}',
                color: '{}',
                size: {},
                font: {{ size: 14, color: '#2c3e50' }}
            }}"#,
            i,
            name,
            role,
            name,
            color,
            if *category == "executive" { 30 } else { 20 }
        ));
    }

    // Create edges
    for (from_id, to_id, rel_type) in relationships {
        if let (Some(&from_idx), Some(&to_idx)) = (id_map.get(&from_id), id_map.get(&to_id)) {
            let arrow_style = match *rel_type {
                "manages" => "to: { enabled: true, type: 'arrow' }",
                "business_relationship" => "to: { enabled: true, type: 'circle' }",
                _ => "to: { enabled: false }",
            };

            edges.push(format!(
                r#"{{
                    from: {},
                    to: {},
                    label: '{}',
                    arrows: {{ {} }},
                    color: {{ color: '#7f8c8d' }},
                    smooth: {{ type: 'continuous' }}
                }}"#,
                from_idx,
                to_idx,
                rel_type,
                arrow_style
            ));
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Person Relationship Network</title>
    <script type="text/javascript" src="https://unpkg.com/vis-network/standalone/umd/vis-network.min.js"></script>
    <style type="text/css">
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        #mynetwork {{
            width: 100%;
            height: 600px;
            border: 2px solid #ddd;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            color: #2c3e50;
            margin-bottom: 20px;
        }}
        .controls {{
            text-align: center;
            margin-top: 20px;
            color: #7f8c8d;
        }}
        .legend {{
            position: absolute;
            top: 40px;
            right: 40px;
            background: white;
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .legend-item {{
            display: flex;
            align-items: center;
            margin-bottom: 8px;
        }}
        .legend-color {{
            width: 20px;
            height: 20px;
            border-radius: 50%;
            margin-right: 10px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Person Relationship Network</h1>
        <p>Interactive visualization of organizational and business relationships</p>
    </div>
    
    <div id="mynetwork"></div>
    
    <div class="legend">
        <h3>Legend</h3>
        <div class="legend-item">
            <div class="legend-color" style="background-color: #ff6b6b;"></div>
            <span>Executive</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background-color: #4ecdc4;"></div>
            <span>Employee</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background-color: #45b7d1;"></div>
            <span>Customer</span>
        </div>
    </div>
    
    <div class="controls">
        <p>🖱️ Drag nodes • 🔍 Scroll to zoom • 👆 Click to select</p>
    </div>

    <script type="text/javascript">
        // Create nodes and edges
        var nodes = new vis.DataSet([
            {}
        ]);

        var edges = new vis.DataSet([
            {}
        ]);

        // Create network
        var container = document.getElementById('mynetwork');
        var data = {{
            nodes: nodes,
            edges: edges
        }};
        
        var options = {{
            nodes: {{
                shape: 'dot',
                borderWidth: 2,
                borderWidthSelected: 4,
                font: {{
                    size: 16,
                    face: 'Arial'
                }}
            }},
            edges: {{
                width: 2,
                font: {{
                    size: 12,
                    align: 'middle'
                }},
                smooth: {{
                    type: 'continuous'
                }}
            }},
            physics: {{
                enabled: true,
                solver: 'forceAtlas2Based',
                forceAtlas2Based: {{
                    gravitationalConstant: -50,
                    centralGravity: 0.01,
                    springLength: 100,
                    springConstant: 0.08
                }},
                stabilization: {{
                    iterations: 200
                }}
            }},
            interaction: {{
                hover: true,
                dragNodes: true,
                zoomView: true,
                dragView: true
            }}
        }};
        
        var network = new vis.Network(container, data, options);
        
        // Add interactivity
        network.on("click", function (params) {{
            if (params.nodes.length > 0) {{
                var nodeId = params.nodes[0];
                var connectedNodes = network.getConnectedNodes(nodeId);
                var allNodes = nodes.get();
                
                // Highlight selected node and connections
                allNodes.forEach(function(node) {{
                    if (node.id === nodeId || connectedNodes.includes(node.id)) {{
                        node.opacity = 1.0;
                    }} else {{
                        node.opacity = 0.3;
                    }}
                }});
                
                nodes.update(allNodes);
            }} else {{
                // Reset opacity
                var allNodes = nodes.get();
                allNodes.forEach(function(node) {{
                    node.opacity = 1.0;
                }});
                nodes.update(allNodes);
            }}
        }});
        
        // Stabilize the network
        network.once('stabilizationIterationsDone', function() {{
            network.setOptions({{ physics: false }});
        }});
    </script>
</body>
</html>"#,
        nodes.join(",\n            "),
        edges.join(",\n            ")
    )
}

fn generate_json_export(
    people: &HashMap<PersonId, (&str, &str, &str)>,
    relationships: &Vec<(PersonId, PersonId, &str)>,
) -> String {
    let mut json = String::from("{\n  \"nodes\": [\n");
    
    let node_entries: Vec<String> = people.iter().map(|(id, (name, role, category))| {
        format!(
            r#"    {{
      "id": "{}",
      "name": "{}",
      "role": "{}",
      "category": "{}"
    }}"#,
            id, name, role, category
        )
    }).collect();
    
    json.push_str(&node_entries.join(",\n"));
    json.push_str("\n  ],\n  \"edges\": [\n");
    
    let edge_entries: Vec<String> = relationships.iter().map(|(from, to, rel_type)| {
        format!(
            r#"    {{
      "from": "{}",
      "to": "{}",
      "relationship": "{}"
    }}"#,
            from, to, rel_type
        )
    }).collect();
    
    json.push_str(&edge_entries.join(",\n"));
    json.push_str("\n  ]\n}");
    
    json
} 