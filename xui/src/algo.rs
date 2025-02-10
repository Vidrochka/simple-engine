use petgraph::{graph::{DiGraph, NodeIndex}, visit::EdgeRef};
use std::collections::{HashMap, VecDeque};

use crate::node::UINode;

pub fn stable_toposort(graph: &DiGraph<UINode, usize>) -> Vec<NodeIndex> {
    let mut in_degree = HashMap::new();
    let mut sorted = Vec::new();
    let mut queue = VecDeque::new();

    // Подсчёт входящих рёбер
    for node in graph.node_indices() {
        in_degree.insert(node, graph.neighbors_directed(node, petgraph::Direction::Incoming).count());
    }

    // Добавляем корневые узлы (без входящих рёбер) в очередь
    let mut roots: Vec<_> = in_degree.iter().filter(|(_, d)| **d == 0).map(|(&n, _)| n).collect();
    roots.sort_by_key(|n| n.index()); // Сортируем по индексу для детерминированности
    queue.extend(roots);

    while let Some(node) = queue.pop_front() {
        sorted.push(node);

        // Получаем всех потомков с их весами
        let mut children: Vec<_> = graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .map(|e| (e.target(), *e.weight())) // (узел, вес рёбра)
            .collect();

        // Сортируем сначала по весу, затем по индексу узла
        children.sort_by_key(|&(n, weight)| (weight, n.index()));

        // Обрабатываем детей
        for (child, _) in children {
            let entry = in_degree.get_mut(&child).unwrap();
            *entry -= 1;
            if *entry == 0 {
                queue.push_back(child);
            }
        }
    }

    sorted
}