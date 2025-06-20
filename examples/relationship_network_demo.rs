//! Relationship Network Visualization Demo
//!
//! This demo creates a visual representation of person relationships,
//! showing organizational hierarchies, business partnerships, and social connections.

use cim_domain_person::{
    aggregate::{Person, PersonId},
    services::PersonCompositionService,
    value_objects::{
        NameComponent, RelationshipComponent, Relationship, RelationshipType, RelationshipStatus,
        SocialMediaComponent, SocialMediaProfile, SocialPlatform, PrivacySetting, SocialMetrics,
        SegmentationComponent, CustomerSegment, ValueTier, LifecycleStage,
        BehavioralComponent, PurchaseBehavior, PredictiveScores,
        SkillsComponent, SkillProficiency, Certification,
        EngagementPatterns, InteractionSummary,
    },
};

use chrono::{NaiveDate, Utc};
use std::collections::HashMap;

fn main() {
    println!("=== Relationship Network Visualization Demo ===\n");

    // Create the service
    let service = PersonCompositionService::new();

    // Create a network of people
    let mut people = HashMap::new();
    let mut relationships = Vec::new();

    // Create CEO
    let ceo_id = PersonId::new();
    let ceo = create_ceo(&service, ceo_id);
    people.insert(ceo_id, ("Victoria Sterling", "CEO", ceo));

    // Create executive team
    let cto_id = PersonId::new();
    let cto = create_executive(&service, cto_id, "Marcus Chen", "CTO", "Technology", Some(ceo_id));
    people.insert(cto_id, ("Marcus Chen", "CTO", cto));
    relationships.push((ceo_id, cto_id, "manages", "organizational"));

    let cfo_id = PersonId::new();
    let cfo = create_executive(&service, cfo_id, "Rachel Goldman", "CFO", "Finance", Some(ceo_id));
    people.insert(cfo_id, ("Rachel Goldman", "CFO", cfo));
    relationships.push((ceo_id, cfo_id, "manages", "organizational"));

    let cmo_id = PersonId::new();
    let cmo = create_executive(&service, cmo_id, "David Park", "CMO", "Marketing", Some(ceo_id));
    people.insert(cmo_id, ("David Park", "CMO", cmo));
    relationships.push((ceo_id, cmo_id, "manages", "organizational"));

    // Create development team
    let dev1_id = PersonId::new();
    let dev1 = create_developer(&service, dev1_id, "Alice Chen", Some(cto_id));
    people.insert(dev1_id, ("Alice Chen", "Sr Developer", dev1));
    relationships.push((cto_id, dev1_id, "manages", "organizational"));

    let dev2_id = PersonId::new();
    let dev2 = create_developer(&service, dev2_id, "Bob Kumar", Some(cto_id));
    people.insert(dev2_id, ("Bob Kumar", "Developer", dev2));
    relationships.push((cto_id, dev2_id, "manages", "organizational"));
    relationships.push((dev1_id, dev2_id, "collaborates", "professional"));

    // Create customers
    let vip_customer_id = PersonId::new();
    let vip_customer = create_vip_customer(&service, vip_customer_id, "TechCorp Inc");
    people.insert(vip_customer_id, ("TechCorp Inc", "VIP Customer", vip_customer));
    relationships.push((cmo_id, vip_customer_id, "account_manager", "business"));

    let influencer_id = PersonId::new();
    let influencer = create_influencer(&service, influencer_id, "Sarah Tech");
    people.insert(influencer_id, ("Sarah Tech", "Influencer", influencer));
    relationships.push((cmo_id, influencer_id, "manages_relationship", "business"));
    relationships.push((influencer_id, vip_customer_id, "referred", "social"));

    // Create business partner
    let partner_id = PersonId::new();
    let partner = create_partner(&service, partner_id, "CloudTech Partners", ceo_id);
    people.insert(partner_id, ("CloudTech CEO", "Partner", partner));
    relationships.push((ceo_id, partner_id, "strategic_partner", "business"));

    // Print network statistics
    print_network_statistics(&people, &relationships);

    // Generate visual representations
    generate_ascii_visualization(&people, &relationships);
    generate_mermaid_diagram(&people, &relationships);
    generate_dot_graph(&people, &relationships);

    // Show relationship insights
    print_relationship_insights(&people, &relationships);

    println!("\n✅ Relationship network visualization complete!");
}

fn create_ceo(service: &PersonCompositionService, person_id: PersonId) -> Person {
    let mut ceo = service.create_employee(
        person_id,
        "Victoria Sterling",
        "Executive",
        Some("Chief Executive Officer"),
        None,
    );

    // Add comprehensive profile
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
                last_active: Some(Utc::now()),
                follower_count: Some(25000),
            },
            SocialMediaProfile {
                platform: SocialPlatform::Twitter,
                username: "vsterling_ceo".to_string(),
                profile_url: Some("https://twitter.com/vsterling_ceo".to_string()),
                verified: true,
                privacy: PrivacySetting::Public,
                last_active: Some(Utc::now()),
                follower_count: Some(12000),
            },
        ],
        metrics: Some(SocialMetrics {
            total_followers: 37000,
            engagement_rate: Some(0.045),
            primary_platform: Some(SocialPlatform::LinkedIn),
            influence_score: Some(88.0),
        }),
    };
    ceo.add_component(social, "Marketing", Some("Executive social media".to_string())).ok();

    ceo
}

fn create_executive(
    service: &PersonCompositionService,
    person_id: PersonId,
    name: &str,
    title: &str,
    department: &str,
    reports_to: Option<PersonId>,
) -> Person {
    service.create_employee(person_id, name, department, Some(title), reports_to)
}

fn create_developer(
    service: &PersonCompositionService,
    person_id: PersonId,
    name: &str,
    reports_to: Option<PersonId>,
) -> Person {
    let mut dev = service.create_employee(
        person_id,
        name,
        "Engineering",
        Some("Software Developer"),
        reports_to,
    );

    // Add technical skills
    let mut skills_map = HashMap::new();
    skills_map.insert("Rust".to_string(), SkillProficiency {
        skill: "Rust".to_string(),
        level: "Expert".to_string(),
        years_experience: Some(3.0),
        last_used: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    });
    skills_map.insert("TypeScript".to_string(), SkillProficiency {
        skill: "TypeScript".to_string(),
        level: "Advanced".to_string(),
        years_experience: Some(4.0),
        last_used: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    });
    skills_map.insert("GraphQL".to_string(), SkillProficiency {
        skill: "GraphQL".to_string(),
        level: "Intermediate".to_string(),
        years_experience: Some(2.0),
        last_used: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    });

    let skills = SkillsComponent {
        skills: skills_map,
        certifications: vec![
            Certification {
                name: "AWS Certified Developer".to_string(),
                issuer: "Amazon Web Services".to_string(),
                issue_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
                expiry_date: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
                credential_id: Some("AWS-DEV-2023".to_string()),
            },
        ],
        education: vec![],
    };
    dev.add_component(skills, "HR", Some("Developer skills".to_string())).ok();

    dev
}

fn create_vip_customer(
    service: &PersonCompositionService,
    person_id: PersonId,
    company: &str,
) -> Person {
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
        lifecycle_stage: LifecycleStage::Advocacy,
        value_tier: ValueTier::Platinum,
        persona: Some("Enterprise".to_string()),
        custom_segments: {
            let mut segments = HashMap::new();
            segments.insert("Industry".to_string(), "Technology".to_string());
            segments.insert("Annual Revenue".to_string(), "$50M+".to_string());
            segments
        },
    };
    customer.add_component(segmentation, "Sales", Some("VIP customer".to_string())).ok();

    // Add behavioral data
    let behavioral = BehavioralComponent {
        purchase_behavior: PurchaseBehavior {
            average_order_value: Some(50000.0),
            purchase_frequency: Some(4.0), // Quarterly
            payment_methods: vec!["Net 30".to_string()],
            seasonal_patterns: HashMap::new(),
            category_distribution: HashMap::new(),
            typical_price_range: Some((25000.0, 100000.0)),
            discount_sensitivity: Some(0.2),
        },
        engagement_patterns: EngagementPatterns {
            email_open_rate: Some(0.85),
            click_through_rate: Some(0.45),
            visit_frequency: Some(8.0),
            avg_session_duration: Some(420),
            device_usage: HashMap::new(),
            active_hours: vec![0.1; 24], // Low activity spread throughout
        },
        interaction_summary: InteractionSummary {
            total_interactions: 150,
            last_interaction: Some(Utc::now()),
            support_tickets: 12,
            avg_satisfaction: Some(4.7),
            channels_used: vec![],
        },
        predictive_scores: PredictiveScores {
            churn_risk: Some(0.05),
            predicted_ltv: Some(2000000.0),
            purchase_probability: Some(0.95),
            upsell_potential: Some(0.80),
            referral_likelihood: Some(0.90),
        },
    };
    customer.add_component(behavioral, "Analytics", Some("VIP behavioral data".to_string())).ok();

    customer
}

fn create_influencer(
    service: &PersonCompositionService,
    person_id: PersonId,
    name: &str,
) -> Person {
    let mut influencer = service.create_customer(
        person_id,
        name,
        Some("sarah@techinfluence.com"),
        Some("+1-555-0200"),
    );

    // Add social media with high follower count
    let social = SocialMediaComponent {
        profiles: vec![
            SocialMediaProfile {
                platform: SocialPlatform::Instagram,
                username: "sarahtech".to_string(),
                profile_url: Some("https://instagram.com/sarahtech".to_string()),
                verified: true,
                privacy: PrivacySetting::Public,
                last_active: Some(Utc::now()),
                follower_count: Some(250000),
            },
            SocialMediaProfile {
                platform: SocialPlatform::YouTube,
                username: "SarahTechReviews".to_string(),
                profile_url: Some("https://youtube.com/@SarahTechReviews".to_string()),
                verified: true,
                privacy: PrivacySetting::Public,
                last_active: Some(Utc::now()),
                follower_count: Some(180000),
            },
            SocialMediaProfile {
                platform: SocialPlatform::Twitter,
                username: "sarah_tech".to_string(),
                profile_url: Some("https://twitter.com/sarah_tech".to_string()),
                verified: true,
                privacy: PrivacySetting::Public,
                last_active: Some(Utc::now()),
                follower_count: Some(95000),
            },
        ],
        metrics: Some(SocialMetrics {
            total_followers: 525000,
            engagement_rate: Some(0.082), // 8.2% - excellent
            primary_platform: Some(SocialPlatform::Instagram),
            influence_score: Some(96.0),
        }),
    };
    influencer.add_component(social, "Marketing", Some("Influencer profile".to_string())).ok();

    influencer
}

fn create_partner(
    service: &PersonCompositionService,
    person_id: PersonId,
    company: &str,
    partner_of: PersonId,
) -> Person {
    let mut partner = service.create_partner(
        person_id,
        &format!("{} CEO", company),
        company,
        Some("Strategic Technology Partner"),
    );

    // Add partnership relationship
    let relationships = RelationshipComponent {
        relationships: vec![
            Relationship {
                person_id: partner_of.into(),
                relationship_type: RelationshipType::BusinessPartner,
                reciprocal_type: RelationshipType::BusinessPartner,
                start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                end_date: None,
                status: RelationshipStatus::Active,
                notes: Some("Strategic cloud infrastructure partnership".to_string()),
            },
        ],
    };
    partner.add_component(relationships, "Partnerships", Some("Partner relationships".to_string())).ok();

    partner
}

fn print_network_statistics(
    people: &HashMap<PersonId, (&str, &str, Person)>,
    relationships: &Vec<(PersonId, PersonId, &str, &str)>,
) {
    println!("📊 Network Statistics:\n");
    println!("  Total People: {}", people.len());
    println!("  Total Relationships: {}", relationships.len());

    // Count by role
    let mut role_counts = HashMap::new();
    for (_, (_, role, _)) in people {
        *role_counts.entry(*role).or_insert(0) += 1;
    }

    println!("\n  People by Role:");
    for (role, count) in role_counts {
        println!("    - {}: {}", role, count);
    }

    // Count by relationship type
    let mut rel_counts = HashMap::new();
    for (_, _, _, rel_type) in relationships {
        *rel_counts.entry(*rel_type).or_insert(0) += 1;
    }

    println!("\n  Relationships by Type:");
    for (rel_type, count) in rel_counts {
        println!("    - {}: {}", rel_type, count);
    }
}

fn generate_ascii_visualization(
    _people: &HashMap<PersonId, (&str, &str, Person)>,
    _relationships: &Vec<(PersonId, PersonId, &str, &str)>,
) {
    println!("\n🎨 ASCII Network Visualization:\n");
    
    println!("                    ┌─────────────────────┐");
    println!("                    │ CEO: Victoria       │");
    println!("                    │ 37K followers       │");
    println!("                    └──────┬──────────────┘");
    println!("                           │");
    println!("         ┌─────────────────┼─────────────────┐");
    println!("         │                 │                 │");
    println!("    ┌────▼────┐      ┌────▼────┐      ┌────▼────┐");
    println!("    │ CTO     │      │ CMO     │      │ CFO     │");
    println!("    │ Marcus  │      │ David   │      │ Rachel  │");
    println!("    └────┬────┘      └────┬────┘      └─────────┘");
    println!("         │                 │");
    println!("    ┌────┴────┐           │");
    println!("    │         │           │");
    println!(" ┌──▼──┐  ┌──▼──┐   ┌────▼────────┐  ┌─────────────┐");
    println!(" │Alice│──│ Bob │   │ VIP Customer │  │ Influencer  │");
    println!(" │Sr.  │  │Dev  │   │ TechCorp    │←─│ 525K follow │");
    println!(" └─────┘  └─────┘   └─────────────┘  └─────────────┘");
    println!("");
    println!("    ┌─────────────────┐");
    println!("    │ Partner: Cloud  │←──── Strategic Partnership");
    println!("    │ Tech CEO        │");
    println!("    └─────────────────┘");
}

fn generate_mermaid_diagram(
    people: &HashMap<PersonId, (&str, &str, Person)>,
    relationships: &Vec<(PersonId, PersonId, &str, &str)>,
) {
    println!("\n📈 Mermaid Diagram (copy to visualizer):\n");
    println!("```mermaid");
    println!("graph TD");
    println!("    classDef executive fill:#ff6b6b,stroke:#333,stroke-width:2px,color:#fff");
    println!("    classDef employee fill:#4ecdc4,stroke:#333,stroke-width:2px");
    println!("    classDef customer fill:#45b7d1,stroke:#333,stroke-width:2px,color:#fff");
    println!("    classDef partner fill:#96ceb4,stroke:#333,stroke-width:2px");
    
    // Add nodes
    for (id, (name, role, person)) in people {
        let node_id = format!("N{}", id.to_string().split('-').next().unwrap());
        let label = format!("{}<br/>{}", name, role);
        
        // Add social metrics if available
        let mut extra_info = String::new();
        if let Some(social) = person.get_component::<SocialMediaComponent>() {
            if let Some(metrics) = &social.metrics {
                extra_info = format!("<br/>{}K followers", metrics.total_followers / 1000);
            }
        }
        
        println!("    {}[\"{}{}\"]::{}", 
            node_id, 
            label, 
            extra_info,
            match *role {
                "CEO" | "CTO" | "CFO" | "CMO" => "executive",
                "Sr Developer" | "Developer" => "employee",
                "VIP Customer" | "Influencer" => "customer",
                "Partner" => "partner",
                _ => "employee",
            }
        );
    }
    
    // Add relationships
    for (from_id, to_id, rel_type, _) in relationships {
        let from_node = format!("N{}", from_id.to_string().split('-').next().unwrap());
        let to_node = format!("N{}", to_id.to_string().split('-').next().unwrap());
        
        let arrow = match *rel_type {
            "manages" => "-->|manages|",
            "strategic_partner" => "<-->|partners|",
            "collaborates" => "---|works with|",
            "referred" => "-.->|referred|",
            _ => "-->|relates|",
        };
        
        println!("    {}{}{}", from_node, arrow, to_node);
    }
    
    println!("```");
}

fn generate_dot_graph(
    people: &HashMap<PersonId, (&str, &str, Person)>,
    relationships: &Vec<(PersonId, PersonId, &str, &str)>,
) {
    println!("\n📊 DOT Graph (for Graphviz):\n");
    println!("digraph RelationshipNetwork {{");
    println!("    rankdir=TB;");
    println!("    node [shape=box, style=rounded];");
    
    // Add nodes with styling
    for (id, (name, role, person)) in people {
        let node_id = format!("N{}", id.to_string().split('-').next().unwrap());
        
        let color = match *role {
            "CEO" | "CTO" | "CFO" | "CMO" => "lightcoral",
            "Sr Developer" | "Developer" => "lightblue",
            "VIP Customer" | "Influencer" => "lightgreen",
            "Partner" => "lightyellow",
            _ => "lightgray",
        };
        
        let mut label = format!("{}\n{}", name, role);
        if let Some(social) = person.get_component::<SocialMediaComponent>() {
            if let Some(metrics) = &social.metrics {
                label.push_str(&format!("\n{}K followers", metrics.total_followers / 1000));
            }
        }
        
        println!("    {} [label=\"{}\", fillcolor={}, style=filled];", 
            node_id, label, color);
    }
    
    // Add edges
    for (from_id, to_id, rel_type, _) in relationships {
        let from_node = format!("N{}", from_id.to_string().split('-').next().unwrap());
        let to_node = format!("N{}", to_id.to_string().split('-').next().unwrap());
        
        let style = match *rel_type {
            "manages" => "solid",
            "strategic_partner" => "bold",
            "collaborates" => "dashed",
            "referred" => "dotted",
            _ => "solid",
        };
        
        println!("    {} -> {} [label=\"{}\", style={}];", 
            from_node, to_node, rel_type, style);
    }
    
    println!("}}");
}

fn print_relationship_insights(
    people: &HashMap<PersonId, (&str, &str, Person)>,
    relationships: &Vec<(PersonId, PersonId, &str, &str)>,
) {
    println!("\n💡 Relationship Insights:\n");
    
    // Find most connected person
    let mut connection_counts = HashMap::new();
    for (from, to, _, _) in relationships {
        *connection_counts.entry(from).or_insert(0) += 1;
        *connection_counts.entry(to).or_insert(0) += 1;
    }
    
    let most_connected = connection_counts.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(id, count)| {
            let (name, role, _) = people.get(id).unwrap();
            (name, role, count)
        });
    
    if let Some((name, role, count)) = most_connected {
        println!("  🌟 Most Connected: {} ({}) with {} connections", name, role, count);
    }
    
    // Find influencers
    println!("\n  📱 Social Media Influencers:");
    for (_, (name, role, person)) in people {
        if let Some(social) = person.get_component::<SocialMediaComponent>() {
            if let Some(metrics) = &social.metrics {
                if metrics.total_followers > 10000 {
                    println!("    - {} ({}): {}K followers, {:.1}% engagement",
                        name, role, 
                        metrics.total_followers / 1000,
                        metrics.engagement_rate.unwrap_or(0.0) * 100.0
                    );
                }
            }
        }
    }
    
    // Business value
    println!("\n  💰 High-Value Relationships:");
    for (_, (name, role, person)) in people {
        if let Some(behavioral) = person.get_component::<BehavioralComponent>() {
            if let Some(ltv) = behavioral.predictive_scores.predicted_ltv {
                if ltv > 100000.0 {
                    println!("    - {} ({}): ${:.0}K predicted LTV", name, role, ltv / 1000.0);
                }
            }
        }
    }
} 