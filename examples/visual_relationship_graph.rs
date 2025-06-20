//! Visual Relationship Graph Demo
//!
//! This demo creates a visual graph showing person relationships,
//! social connections, and organizational structures using the CIM graph domain.

use cim_domain_person::{
    aggregate::PersonId,
    services::PersonCompositionService,
    value_objects::{
        NameComponent, RelationshipComponent, Relationship, RelationshipType, RelationshipStatus,
        SocialMediaComponent, SocialMediaProfile, SocialPlatform, PrivacySetting,
        EmploymentComponent, SegmentationComponent, CustomerSegment, ValueTier,
    },
};

use cim_domain_graph::{
    aggregate::{GraphId, GraphAggregate},
    commands::GraphCommand,
    value_objects::{NodeType, EdgeType, Position3D, NodeMetadata, EdgeMetadata},
};

use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("=== Visual Relationship Graph Demo ===\n");

    // Create a graph to visualize relationships
    let mut graph = GraphAggregate::new(
        GraphId::new(),
        "Person Relationship Network".to_string(),
        cim_domain_graph::value_objects::GraphType::SocialNetwork,
    );

    // Create the service
    let service = PersonCompositionService::new();

    // Create central person - CEO
    let ceo_id = PersonId::new();
    let ceo = create_ceo(&service, ceo_id);
    
    // Add CEO to graph
    add_person_to_graph(&mut graph, &ceo, Position3D::new(0.0, 0.0, 0.0), "CEO");

    // Create executive team
    let cto_id = PersonId::new();
    let cto = create_cto(&service, cto_id, ceo_id);
    add_person_to_graph(&mut graph, &cto, Position3D::new(-150.0, -100.0, 0.0), "CTO");

    let cfo_id = PersonId::new();
    let cfo = create_cfo(&service, cfo_id, ceo_id);
    add_person_to_graph(&mut graph, &cfo, Position3D::new(150.0, -100.0, 0.0), "CFO");

    let cmo_id = PersonId::new();
    let cmo = create_cmo(&service, cmo_id, ceo_id);
    add_person_to_graph(&mut graph, &cmo, Position3D::new(0.0, -100.0, 0.0), "CMO");

    // Create team members
    let dev1_id = PersonId::new();
    let dev1 = create_developer(&service, dev1_id, cto_id, "Alice Chen");
    add_person_to_graph(&mut graph, &dev1, Position3D::new(-200.0, -200.0, 0.0), "Sr Dev");

    let dev2_id = PersonId::new();
    let dev2 = create_developer(&service, dev2_id, cto_id, "Bob Kumar");
    add_person_to_graph(&mut graph, &dev2, Position3D::new(-100.0, -200.0, 0.0), "Dev");

    // Create key customers
    let customer1_id = PersonId::new();
    let customer1 = create_vip_customer(&service, customer1_id, "TechCorp Inc");
    add_person_to_graph(&mut graph, &customer1, Position3D::new(-50.0, 150.0, 0.0), "VIP Customer");

    let customer2_id = PersonId::new();
    let customer2 = create_influencer_customer(&service, customer2_id, "Sarah Influencer");
    add_person_to_graph(&mut graph, &customer2, Position3D::new(50.0, 150.0, 0.0), "Influencer");

    // Create business partners
    let partner1_id = PersonId::new();
    let partner1 = create_partner(&service, partner1_id, "CloudTech Partners", ceo_id);
    add_person_to_graph(&mut graph, &partner1, Position3D::new(250.0, 0.0, 0.0), "Partner");

    // Add relationships as edges
    println!("📊 Building Relationship Graph...\n");

    // Organizational hierarchy
    add_relationship_edge(&mut graph, ceo_id, cto_id, "Reports To", EdgeType::Organizational);
    add_relationship_edge(&mut graph, ceo_id, cfo_id, "Reports To", EdgeType::Organizational);
    add_relationship_edge(&mut graph, ceo_id, cmo_id, "Reports To", EdgeType::Organizational);
    add_relationship_edge(&mut graph, cto_id, dev1_id, "Manages", EdgeType::Organizational);
    add_relationship_edge(&mut graph, cto_id, dev2_id, "Manages", EdgeType::Organizational);

    // Business relationships
    add_relationship_edge(&mut graph, ceo_id, partner1_id, "Strategic Partner", EdgeType::Business);
    add_relationship_edge(&mut graph, cmo_id, customer1_id, "Account Manager", EdgeType::Business);
    add_relationship_edge(&mut graph, cmo_id, customer2_id, "Influencer Relations", EdgeType::Business);

    // Social connections
    add_relationship_edge(&mut graph, dev1_id, dev2_id, "Collaborates With", EdgeType::Social);
    add_relationship_edge(&mut graph, customer2_id, customer1_id, "Referred", EdgeType::Social);

    // Print graph statistics
    print_graph_statistics(&graph);

    // Generate visual representation
    generate_visual_output(&graph);

    println!("\n✅ Visual relationship graph created successfully!");
}

fn create_ceo(service: &PersonCompositionService, person_id: PersonId) -> cim_domain_person::aggregate::Person {
    let mut ceo = service.create_employee(
        person_id,
        "Victoria Sterling",
        "Executive",
        Some("Chief Executive Officer"),
        None,
    );

    // Add comprehensive name
    let name = NameComponent {
        title: Some("Ms.".to_string()),
        honorific: None,
        given_names: vec!["Victoria".to_string()],
        middle_names: vec!["Elizabeth".to_string()],
        family_names: vec!["Sterling".to_string()],
        maternal_family_name: None,
        generational_suffix: None,
        professional_suffix: Some("MBA".to_string()),
        preferred_name: Some("Victoria".to_string()),
        name_order: cim_domain_person::value_objects::NameOrder::GivenFirst,
        cultural_context: None,
    };
    ceo.add_component(name, "HR", Some("Executive profile".to_string())).ok();

    // Add social media presence
    let social = SocialMediaComponent {
        profiles: vec![
            SocialMediaProfile {
                platform: SocialPlatform::LinkedIn,
                username: "vstirling".to_string(),
                profile_url: Some("https://linkedin.com/in/vstirling".to_string()),
                verified: true,
                privacy: PrivacySetting::Professional,
                last_active: Some(chrono::Utc::now()),
                follower_count: Some(25000),
            },
        ],
        metrics: None,
    };
    ceo.add_component(social, "Marketing", Some("Executive social media".to_string())).ok();

    ceo
}

fn create_cto(service: &PersonCompositionService, person_id: PersonId, reports_to: PersonId) -> cim_domain_person::aggregate::Person {
    let cto = service.create_employee(
        person_id,
        "Marcus Chen",
        "Technology",
        Some("Chief Technology Officer"),
        Some(reports_to),
    );
    cto
}

fn create_cfo(service: &PersonCompositionService, person_id: PersonId, reports_to: PersonId) -> cim_domain_person::aggregate::Person {
    let cfo = service.create_employee(
        person_id,
        "Rachel Goldman",
        "Finance",
        Some("Chief Financial Officer"),
        Some(reports_to),
    );
    cfo
}

fn create_cmo(service: &PersonCompositionService, person_id: PersonId, reports_to: PersonId) -> cim_domain_person::aggregate::Person {
    let cmo = service.create_employee(
        person_id,
        "David Park",
        "Marketing",
        Some("Chief Marketing Officer"),
        Some(reports_to),
    );
    cmo
}

fn create_developer(service: &PersonCompositionService, person_id: PersonId, reports_to: PersonId, name: &str) -> cim_domain_person::aggregate::Person {
    let dev = service.create_employee(
        person_id,
        name,
        "Engineering",
        Some("Software Developer"),
        Some(reports_to),
    );
    dev
}

fn create_vip_customer(service: &PersonCompositionService, person_id: PersonId, company: &str) -> cim_domain_person::aggregate::Person {
    let mut customer = service.create_customer(
        person_id,
        company,
        Some(&format!("contact@{}.com", company.to_lowercase().replace(" ", ""))),
        Some("+1-555-0100"),
    );

    // Add VIP segmentation
    let segmentation = SegmentationComponent {
        primary_segment: CustomerSegment::VIPCustomer,
        secondary_segments: vec![CustomerSegment::LoyalCustomer],
        lifecycle_stage: cim_domain_person::value_objects::LifecycleStage::Advocacy,
        value_tier: ValueTier::Platinum,
        persona: Some("Enterprise".to_string()),
        custom_segments: HashMap::new(),
    };
    customer.add_component(segmentation, "Sales", Some("VIP customer".to_string())).ok();

    customer
}

fn create_influencer_customer(service: &PersonCompositionService, person_id: PersonId, name: &str) -> cim_domain_person::aggregate::Person {
    let mut customer = service.create_customer(
        person_id,
        name,
        Some("sarah@influence.com"),
        Some("+1-555-0200"),
    );

    // Add social media
    let social = SocialMediaComponent {
        profiles: vec![
            SocialMediaProfile {
                platform: SocialPlatform::Instagram,
                username: "sarahtech".to_string(),
                profile_url: Some("https://instagram.com/sarahtech".to_string()),
                verified: true,
                privacy: PrivacySetting::Public,
                last_active: Some(chrono::Utc::now()),
                follower_count: Some(150000),
            },
        ],
        metrics: Some(cim_domain_person::value_objects::SocialMetrics {
            total_followers: 150000,
            engagement_rate: Some(0.08),
            primary_platform: Some(SocialPlatform::Instagram),
            influence_score: Some(95.0),
        }),
    };
    customer.add_component(social, "Marketing", Some("Influencer profile".to_string())).ok();

    customer
}

fn create_partner(service: &PersonCompositionService, person_id: PersonId, company: &str, partner_of: PersonId) -> cim_domain_person::aggregate::Person {
    let mut partner = service.create_partner(
        person_id,
        &format!("{} CEO", company),
        company,
        Some("Strategic Technology Partner"),
    );

    // Add relationship to CEO
    let relationships = RelationshipComponent {
        relationships: vec![
            Relationship {
                person_id: partner_of.into(),
                relationship_type: RelationshipType::BusinessPartner,
                reciprocal_type: RelationshipType::BusinessPartner,
                start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                end_date: None,
                status: RelationshipStatus::Active,
                notes: Some("Strategic partnership agreement".to_string()),
            },
        ],
    };
    partner.add_component(relationships, "Partnerships", Some("Partner relationships".to_string())).ok();

    partner
}

fn add_person_to_graph(
    graph: &mut GraphAggregate,
    person: &cim_domain_person::aggregate::Person,
    position: Position3D,
    role: &str,
) {
    let node_id = cim_domain_graph::value_objects::NodeId::from(person.id().into());
    
    let mut metadata = HashMap::new();
    metadata.insert("role".to_string(), role.to_string());
    metadata.insert("name".to_string(), person.get_identity().legal_name.clone());
    
    if let Some(employment) = person.get_component::<EmploymentComponent>() {
        metadata.insert("department".to_string(), employment.department.clone().unwrap_or_default());
        metadata.insert("title".to_string(), employment.title.clone());
    }

    let node_type = match role {
        "CEO" | "CTO" | "CFO" | "CMO" => NodeType::Executive,
        "Sr Dev" | "Dev" => NodeType::Employee,
        "VIP Customer" | "Influencer" => NodeType::Customer,
        "Partner" => NodeType::Partner,
        _ => NodeType::Person,
    };

    graph.handle_command(GraphCommand::AddNode {
        node_id,
        node_type,
        position,
        metadata: NodeMetadata(metadata),
    }).ok();
}

fn add_relationship_edge(
    graph: &mut GraphAggregate,
    from_person: PersonId,
    to_person: PersonId,
    label: &str,
    edge_type: EdgeType,
) {
    let source = cim_domain_graph::value_objects::NodeId::from(from_person.into());
    let target = cim_domain_graph::value_objects::NodeId::from(to_person.into());
    
    let mut metadata = HashMap::new();
    metadata.insert("label".to_string(), label.to_string());
    metadata.insert("strength".to_string(), "1.0".to_string());

    graph.handle_command(GraphCommand::ConnectNodes {
        source,
        target,
        edge_type,
        metadata: Some(EdgeMetadata(metadata)),
    }).ok();
}

fn print_graph_statistics(graph: &GraphAggregate) {
    println!("📈 Graph Statistics:");
    println!("  Nodes: {}", graph.nodes().len());
    println!("  Edges: {}", graph.edges().len());
    println!("  Node Types:");
    
    let mut type_counts = HashMap::new();
    for node in graph.nodes().values() {
        *type_counts.entry(format!("{:?}", node.node_type)).or_insert(0) += 1;
    }
    
    for (node_type, count) in type_counts {
        println!("    - {}: {}", node_type, count);
    }
    
    println!("\n  Edge Types:");
    let mut edge_counts = HashMap::new();
    for edge in graph.edges().values() {
        *edge_counts.entry(format!("{:?}", edge.edge_type)).or_insert(0) += 1;
    }
    
    for (edge_type, count) in edge_counts {
        println!("    - {}: {}", edge_type, count);
    }
}

fn generate_visual_output(graph: &GraphAggregate) {
    println!("\n🎨 Visual Representation:\n");
    
    // Generate ASCII art representation
    println!("                    [CEO: Victoria Sterling]");
    println!("                           /    |    \\");
    println!("                          /     |     \\");
    println!("                         /      |      \\");
    println!("            [CTO: Marcus]   [CMO: David]   [CFO: Rachel]");
    println!("                 / \\            |");
    println!("                /   \\           |");
    println!("        [Alice] --- [Bob]   [Customers]");
    println!("                              / \\");
    println!("                             /   \\");
    println!("                    [TechCorp] [Influencer]");
    println!("                                    |");
    println!("                              [150K followers]");
    println!("\n        [Partner: CloudTech] ←→ CEO");
    
    println!("\n📍 Legend:");
    println!("  ─── Organizational hierarchy");
    println!("  ←→  Business partnership");
    println!("  --- Team collaboration");
    
    // Generate Mermaid diagram
    println!("\n📊 Mermaid Diagram (copy to visualizer):\n");
    println!("```mermaid");
    println!("graph TD");
    
    // Add nodes with styling
    for (node_id, node) in graph.nodes() {
        let name = node.metadata.0.get("name").unwrap_or(&"Unknown".to_string());
        let role = node.metadata.0.get("role").unwrap_or(&"".to_string());
        let node_label = format!("{}<br/>{}", name, role);
        
        let style = match node.node_type {
            NodeType::Executive => "fill:#ff6b6b,stroke:#333,stroke-width:2px",
            NodeType::Employee => "fill:#4ecdc4,stroke:#333,stroke-width:2px",
            NodeType::Customer => "fill:#45b7d1,stroke:#333,stroke-width:2px",
            NodeType::Partner => "fill:#96ceb4,stroke:#333,stroke-width:2px",
            _ => "fill:#dfe6e9,stroke:#333,stroke-width:2px",
        };
        
        println!("    {}[{}]", node_id.to_string().split('-').next().unwrap(), node_label);
        println!("    style {} {}", node_id.to_string().split('-').next().unwrap(), style);
    }
    
    // Add edges
    for edge in graph.edges().values() {
        let label = edge.metadata.as_ref()
            .and_then(|m| m.0.get("label"))
            .unwrap_or(&"".to_string());
        
        let arrow = match edge.edge_type {
            EdgeType::Organizational => "-->",
            EdgeType::Business => "<-->",
            EdgeType::Social => "---",
            _ => "-->",
        };
        
        println!("    {}{}{}{}", 
            edge.source.to_string().split('-').next().unwrap(),
            arrow,
            if label.is_empty() { "" } else { "|" },
            if label.is_empty() { "" } else { label }
        );
        println!("    {}", edge.target.to_string().split('-').next().unwrap());
    }
    
    println!("```");
}

// Define edge types for relationships
#[derive(Debug, Clone, PartialEq)]
enum EdgeType {
    Organizational,
    Business,
    Social,
}

// Define node types for people
#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Executive,
    Employee,
    Customer,
    Partner,
    Person,
} 