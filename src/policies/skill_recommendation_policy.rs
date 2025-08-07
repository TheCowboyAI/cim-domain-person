//! Policy for recommending skills based on activity

use async_trait::async_trait;
use cim_domain::DomainResult;

use crate::aggregate::ComponentType;
use crate::commands::{PersonCommand, AddComponent};
use crate::events::PersonEventV2;
use super::Policy;

/// Policy that recommends skills based on person's activities
pub struct SkillRecommendationPolicy;

impl Default for SkillRecommendationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRecommendationPolicy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Policy for SkillRecommendationPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        match event {
            PersonEventV2::ComponentAdded { person_id, component_type, component_data, .. } => {
                // If a Git profile is added, recommend programming skills
                if matches!(component_type, ComponentType::CustomAttribute) {
                    if let Some(comp_type) = component_data.get("type").and_then(|v| v.as_str()) {
                        if comp_type == "git_profile" {
                            let languages = component_data.get("languages")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                )
                                .unwrap_or_default();
                            
                            if !languages.is_empty() {
                                // Generate skill recommendations
                                let recommendations = generate_skill_recommendations(&languages);
                                
                                return Ok(vec![
                                    PersonCommand::AddComponent(AddComponent {
                                        person_id: *person_id,
                                        component_type: ComponentType::CustomAttribute,
                                        data: serde_json::json!({
                                            "type": "skill_recommendations",
                                            "recommendations": recommendations,
                                            "source": "git_profile_analysis",
                                            "generated_at": chrono::Utc::now(),
                                        }),
                                    })
                                ]);
                            }
                        }
                    }
                }
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }
    
    fn name(&self) -> &str {
        "SkillRecommendation"
    }
    
    fn applies_to(&self, event: &PersonEventV2) -> bool {
        matches!(event, PersonEventV2::ComponentAdded { .. })
    }
}

fn generate_skill_recommendations(languages: &[&str]) -> Vec<serde_json::Value> {
    let mut recommendations = Vec::new();
    
    for lang in languages {
        match *lang {
            "rust" => {
                recommendations.push(serde_json::json!({
                    "skill": "Systems Programming",
                    "confidence": 0.9,
                    "related_skills": ["Memory Management", "Concurrent Programming"]
                }));
                recommendations.push(serde_json::json!({
                    "skill": "WebAssembly",
                    "confidence": 0.7,
                    "related_skills": ["Web Development", "Performance Optimization"]
                }));
            }
            "python" => {
                recommendations.push(serde_json::json!({
                    "skill": "Data Science",
                    "confidence": 0.8,
                    "related_skills": ["Machine Learning", "Data Analysis"]
                }));
                recommendations.push(serde_json::json!({
                    "skill": "Automation",
                    "confidence": 0.85,
                    "related_skills": ["DevOps", "Scripting"]
                }));
            }
            "javascript" | "typescript" => {
                recommendations.push(serde_json::json!({
                    "skill": "Web Development",
                    "confidence": 0.95,
                    "related_skills": ["Frontend", "Node.js", "React"]
                }));
            }
            _ => {}
        }
    }
    
    recommendations
}