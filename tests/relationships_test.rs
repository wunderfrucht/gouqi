use gouqi::relationships::{RelationshipGraph, IssueRelationships, GraphOptions};

#[test]
fn test_relationship_graph_creation() {
    let graph = RelationshipGraph::new("test".to_string());
    
    assert_eq!(graph.metadata.source, "test");
    assert_eq!(graph.metadata.issue_count, 0);
    assert_eq!(graph.metadata.relationship_count, 0);
    assert!(graph.issues.is_empty());
}

#[test]
fn test_add_issue_to_graph() {
    let mut graph = RelationshipGraph::new("test".to_string());
    let mut relationships = IssueRelationships::new();
    relationships.blocks.push("PROJ-124".to_string());
    relationships.relates_to.push("PROJ-125".to_string());
    
    graph.add_issue("PROJ-123".to_string(), relationships);
    
    assert_eq!(graph.metadata.issue_count, 1);
    assert_eq!(graph.metadata.relationship_count, 2);
    assert!(graph.contains_issue("PROJ-123"));
    
    let retrieved_relationships = graph.get_relationships("PROJ-123").unwrap();
    assert_eq!(retrieved_relationships.blocks.len(), 1);
    assert_eq!(retrieved_relationships.blocks[0], "PROJ-124");
    assert_eq!(retrieved_relationships.relates_to.len(), 1);
    assert_eq!(retrieved_relationships.relates_to[0], "PROJ-125");
}

#[test]
fn test_issue_relationships_operations() {
    let mut relationships = IssueRelationships::new();
    
    // Test adding relationships
    relationships.add_relationship("blocks", "PROJ-124".to_string());
    relationships.add_relationship("blocked_by", "PROJ-122".to_string());
    relationships.add_relationship("parent", "PROJ-100".to_string());
    relationships.add_relationship("custom_implements", "PROJ-200".to_string());
    
    assert_eq!(relationships.blocks.len(), 1);
    assert_eq!(relationships.blocks[0], "PROJ-124");
    assert_eq!(relationships.blocked_by.len(), 1);
    assert_eq!(relationships.blocked_by[0], "PROJ-122");
    assert_eq!(relationships.parent, Some("PROJ-100".to_string()));
    assert_eq!(relationships.custom.get("custom_implements").unwrap().len(), 1);
    
    // Test removing relationships
    relationships.remove_relationship("blocks", "PROJ-124");
    relationships.remove_relationship("parent", "PROJ-100");
    relationships.remove_relationship("custom_implements", "PROJ-200");
    
    assert!(relationships.blocks.is_empty());
    assert_eq!(relationships.parent, None);
    assert!(relationships.custom.is_empty());
    
    assert!(!relationships.is_empty()); // still has blocked_by
}

#[test]
fn test_get_all_related() {
    let mut relationships = IssueRelationships::new();
    relationships.blocks.push("PROJ-124".to_string());
    relationships.blocked_by.push("PROJ-122".to_string());
    relationships.relates_to.push("PROJ-125".to_string());
    relationships.parent = Some("PROJ-100".to_string());
    relationships.epic = Some("PROJ-50".to_string());
    relationships.children.push("PROJ-126".to_string());
    
    let related = relationships.get_all_related();
    
    assert_eq!(related.len(), 6);
    assert!(related.contains("PROJ-124"));
    assert!(related.contains("PROJ-122"));
    assert!(related.contains("PROJ-125"));
    assert!(related.contains("PROJ-100"));
    assert!(related.contains("PROJ-50"));
    assert!(related.contains("PROJ-126"));
}

#[test]
fn test_graph_path_finding() {
    let mut graph = RelationshipGraph::new("test".to_string());
    
    // Create a simple chain: A -> B -> C
    let mut rel_a = IssueRelationships::new();
    rel_a.blocks.push("PROJ-B".to_string());
    graph.add_issue("PROJ-A".to_string(), rel_a);
    
    let mut rel_b = IssueRelationships::new();
    rel_b.blocked_by.push("PROJ-A".to_string());
    rel_b.blocks.push("PROJ-C".to_string());
    graph.add_issue("PROJ-B".to_string(), rel_b);
    
    let mut rel_c = IssueRelationships::new();
    rel_c.blocked_by.push("PROJ-B".to_string());
    graph.add_issue("PROJ-C".to_string(), rel_c);
    
    // Test path finding
    let path = graph.get_path("PROJ-A", "PROJ-C");
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 3);
    assert_eq!(path[0], "PROJ-A");
    assert_eq!(path[1], "PROJ-B");
    assert_eq!(path[2], "PROJ-C");
    
    // Test direct path
    let direct_path = graph.get_path("PROJ-A", "PROJ-A");
    assert!(direct_path.is_some());
    assert_eq!(direct_path.unwrap(), vec!["PROJ-A"]);
    
    // Test no path
    let rel_d = IssueRelationships::new();
    graph.add_issue("PROJ-D".to_string(), rel_d);
    let no_path = graph.get_path("PROJ-A", "PROJ-D");
    assert!(no_path.is_none());
}

#[test]
fn test_graph_options() {
    let options = GraphOptions::default();
    assert!(options.include_custom);
    assert!(options.bidirectional);
    assert!(options.include_types.is_none());
    assert!(options.exclude_types.is_none());
    
    let custom_options = GraphOptions {
        include_types: Some(vec!["blocks".to_string(), "relates_to".to_string()]),
        exclude_types: Some(vec!["duplicates".to_string()]),
        include_custom: false,
        bidirectional: false,
    };
    
    assert!(!custom_options.include_custom);
    assert!(!custom_options.bidirectional);
    assert_eq!(custom_options.include_types.as_ref().unwrap().len(), 2);
    assert_eq!(custom_options.exclude_types.as_ref().unwrap().len(), 1);
}

#[test]
fn test_relationship_serialization() {
    let mut graph = RelationshipGraph::new("test".to_string());
    let mut relationships = IssueRelationships::new();
    
    relationships.blocks.push("PROJ-124".to_string());
    relationships.blocked_by.push("PROJ-122".to_string());
    relationships.parent = Some("PROJ-100".to_string());
    relationships.custom.insert("implements".to_string(), vec!["PROJ-200".to_string()]);
    
    graph.add_issue("PROJ-123".to_string(), relationships);
    
    // Test JSON serialization
    let json = serde_json::to_string(&graph).expect("Should serialize to JSON");
    assert!(json.contains("PROJ-123"));
    assert!(json.contains("PROJ-124"));
    assert!(json.contains("implements"));
    
    // Test deserialization
    let deserialized: RelationshipGraph = serde_json::from_str(&json)
        .expect("Should deserialize from JSON");
    
    assert_eq!(deserialized.metadata.source, "test");
    assert_eq!(deserialized.metadata.issue_count, 1);
    assert!(deserialized.contains_issue("PROJ-123"));
    
    let rel = deserialized.get_relationships("PROJ-123").unwrap();
    assert_eq!(rel.blocks[0], "PROJ-124");
    assert_eq!(rel.blocked_by[0], "PROJ-122");
    assert_eq!(rel.parent.as_ref().unwrap(), "PROJ-100");
    assert_eq!(rel.custom.get("implements").unwrap()[0], "PROJ-200");
}

#[test]
fn test_empty_relationships() {
    let relationships = IssueRelationships::new();
    assert!(relationships.is_empty());
    
    let mut non_empty = IssueRelationships::new();
    non_empty.blocks.push("PROJ-123".to_string());
    assert!(!non_empty.is_empty());
}

#[test]
fn test_graph_metadata_updates() {
    let mut graph = RelationshipGraph::new("test".to_string());
    
    // Add first issue
    let mut rel1 = IssueRelationships::new();
    rel1.blocks.push("PROJ-B".to_string());
    rel1.relates_to.push("PROJ-C".to_string());
    graph.add_issue("PROJ-A".to_string(), rel1);
    
    assert_eq!(graph.metadata.issue_count, 1);
    assert_eq!(graph.metadata.relationship_count, 2);
    
    // Add second issue
    let mut rel2 = IssueRelationships::new();
    rel2.blocked_by.push("PROJ-A".to_string());
    rel2.parent = Some("PROJ-PARENT".to_string());
    graph.add_issue("PROJ-B".to_string(), rel2);
    
    assert_eq!(graph.metadata.issue_count, 2);
    assert_eq!(graph.metadata.relationship_count, 4);
}