use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};

peg::parser! {
  pub grammar c4lang() for str {
    pub rule identifier() -> String
      = n:$([^'0'..='9']['0'..='9'|'a'..='z' | 'A'..='Z' | '_' ]+) {? Ok(n.to_string()) };

    rule quoted_string() -> &'input str = "\"" n:$([^'"']+) "\"" { n }
    rule non_ws() -> &'input str = n:$([^' ']+)

    pub rule string() -> String =
        "\"" n:$([^'"']+) "\"" { n.to_string() }
        / n:$([^' ']+) { n.to_string() }

    pub rule whitespaces() -> &'input str =
        n: $([' ']*) { n }

    pub rule all_ws() -> &'input str =
        n: $([' ' | '\n' | '\t']*) { n }

    pub rule name_assignment() -> (String, String)
        = i: identifier() ".name" whitespaces() "=" whitespaces() v: string() {
            (i, v)
        }
    pub rule point_to() -> Option<String> =
        ['-']+ i: identifier() ['-']+ ">" { Option::Some(i) }
        / ['-']+ ">" { Option::None };

    pub rule relationship() -> CArrow = whitespaces()? a: identifier() whitespaces()? p: point_to() whitespaces()? b: identifier() whitespaces()? {
        CArrow {
            name: p,
            from: CBox { name: a },
            to: CBox { name: b },
        }
    }

    pub rule all_relations() -> Vec<CArrow> = all_ws() i: (relationship() ++ "\n") all_ws() ![_]?{ i }
  }
}

#[derive(Debug, PartialEq)]
pub struct CBox {
    name: String,
}

#[derive(Debug, PartialEq)]
pub struct CArrow {
    name: Option<String>,
    from: CBox,
    to: CBox,
}

pub fn parse_to_graph(text: &str) -> Option<DiGraph<String, Option<String>>> {
    let relationships = c4lang::all_relations(text);
    let mut graph = DiGraph::<String, Option<String>>::new();
    return relationships
        .map(|arrows| {
            let mut nodes_seen: HashMap<String, NodeIndex> = HashMap::new();
            arrows.iter().for_each(|ar| {
                let from_id = ar.from.name.clone();
                let to_id = ar.to.name.clone();

                let f = if let Some(f_id) = nodes_seen.get(from_id.as_str()) {
                    *f_id
                } else {
                    let id = graph.add_node(from_id.clone());
                    nodes_seen.insert(from_id, id);
                    id
                };

                let t = if let Some(t_id) = nodes_seen.get(to_id.as_str()) {
                    *t_id
                } else {
                    let id = graph.add_node(to_id.clone());
                    nodes_seen.insert(to_id, id);
                    id
                };

                graph.add_edge(f, t, ar.name.clone());
            });
        })
        .ok()
        .map(|_| graph);
}

#[test]
fn test_parser() {
    assert!(c4lang::identifier("11aa2").is_err());
    assert_eq!(
        c4lang::string(r#""mamamia is a 1002.jjd££4""#),
        Ok("mamamia is a 1002.jjd££4".to_string())
    );
    assert_eq!(
        c4lang::string(r#"mamamia££4"#),
        Ok("mamamia££4".to_string())
    );
    let mut o = CArrow {
        name: None,
        from: CBox {
            name: "cow".to_string(),
        },
        to: CBox {
            name: "fresh".to_string(),
        },
    };
    assert_eq!(
        c4lang::relationship(r#"cow -------> fresh"#).as_ref(),
        Ok(&o)
    );

    o.name = Some("hoho".to_string());
    assert_eq!(
        c4lang::relationship(r#"cow ---hoho----> fresh"#).as_ref(),
        Ok(&o)
    );

    let o2 = CArrow {
        name: Some("hoho".to_string()),
        from: CBox {
            name: "cow2".to_string(),
        },
        to: CBox {
            name: "fresh2".to_string(),
        },
    };
    assert_eq!(
        c4lang::all_relations(
            r#"
    cow ---hoho----> fresh
    cow2 ---hoho----> fresh2
    "#
        ),
        Ok(vec![o, o2])
    );
}
