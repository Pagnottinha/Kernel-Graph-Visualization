use eframe::{egui, run_native, App, CreationContext, NativeOptions};
use egui::{Color32, FontId, Pos2, Shape, Vec2};
use egui_graphs::{
    DefaultEdgeShape, DisplayNode, DrawContext, NodeProps, 
    FruchtermanReingold, FruchtermanReingoldState, 
    FruchtermanReingoldWithCenterGravity, FruchtermanReingoldWithCenterGravityState, 
    Graph, GraphView, LayoutForceDirected, SettingsNavigation, SettingsStyle,
    LayoutState
};
use petgraph::{Directed, graph::IndexType, stable_graph::{NodeIndex, StableGraph}};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct CustomNodeShape {
    location: Pos2,
    radius: f32,
    label: String,
    selected: bool,
    dragged: bool,
    hovered: bool,
}

impl<N: Clone> From<NodeProps<N>> for CustomNodeShape {
    fn from(props: NodeProps<N>) -> Self {
        Self {
            location: props.location(),
            radius: 3.0,
            label: props.label.clone(),
            selected: props.selected,
            dragged: props.dragged,
            hovered: props.hovered,
        }
    }
}

impl<N: Clone, E: Clone, Ty: petgraph::EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix> for CustomNodeShape {
    
    fn update(&mut self, props: &NodeProps<N>) {
        self.location = props.location();
        self.label = props.label.clone();
        self.selected = props.selected;
        self.dragged = props.dragged;
        self.hovered = props.hovered;
        self.radius = 3.0;
    }

    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        self.location + dir.normalized() * self.radius
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        (pos - self.location).length() <= self.radius * 30.0
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut color = Color32::from_rgb(90, 170, 255);

        let screen_loc = ctx.meta.canvas_to_screen_pos(self.location);

        if self.dragged {
            color = Color32::from_rgb(255, 100, 100);
        } else if self.selected {
            color = Color32::from_rgb(255, 200, 50);
        } else if self.hovered {
            color = Color32::from_rgb(150, 210, 255);
        }

        let circle = Shape::circle_filled(screen_loc, self.radius, color);

        let font_id = FontId::proportional(3.0 * self.radius);
        
        let galley = ctx.painter.layout_no_wrap(self.label.clone(), font_id, Color32::WHITE);

        let text_pos = screen_loc + Vec2::new(-galley.size().x / 2.0, self.radius + 4.0);
        let text_shape = Shape::galley(text_pos, galley, Color32::WHITE);

        vec![circle, text_shape]
    }
}

pub struct KernelGraphApp {
    g: Graph<(), (), Directed, u32, CustomNodeShape, DefaultEdgeShape>,
    auto_fit: bool,
    physics_initialized: bool,
    filter_text: String,
    highlight_text: String,
}

impl KernelGraphApp {
pub fn new(_cc: &CreationContext) -> Self {
        let mut app = Self {
            g: Graph::new(StableGraph::new()),
            auto_fit: false,
            physics_initialized: false,
            filter_text: "adc/".to_string(),
            highlight_text: "".to_string(),
        };
        

        app.reload_graph();
        app
    }

    pub fn reload_graph(&mut self) {
        if let Ok((stable_g, labels)) = generate_graph(&self.filter_text) {

            let mut new_g = Graph::from(&stable_g);

            let mut angle: f32 = 0.0;
            let mut radius: f32 = 50.0;

            for (idx, text) in labels {
                if let Some(node) = new_g.node_mut(idx) {
                    node.set_label(text);
                    let x = radius * angle.cos();
                    let y = radius * angle.sin();
                    node.set_location(egui::pos2(x, y));

                    angle += 2.4;
                    radius += 15.0;
                }
            }

            self.g = new_g;
            self.physics_initialized = false;
        }
    }
}

impl App for KernelGraphApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Vizualization in Graph");
                ui.checkbox(&mut self.auto_fit, "Auto-fit Cam");
            });
            
            ui.horizontal(|ui| {
                ui.label("Directories (split them with commas):");
                ui.text_edit_singleline(&mut self.filter_text);

                if ui.button("Filter and recreate").clicked() {
                    self.reload_graph();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Highlight files:");
                let response = ui.text_edit_singleline(&mut self.highlight_text);
            
                if response.changed() {
                    let query = self.highlight_text.trim().to_lowercase();
                    let mut selected_nodes = HashSet::new();


                    let node_indices: Vec<_> = self.g.g().node_indices().collect();


                    for idx in node_indices {

                        if let Some(node) = self.g.node_mut(idx) {
                            if !query.is_empty() && node.label().to_lowercase().contains(&query) {
                                node.set_selected(true);
                                selected_nodes.insert(idx);
                            } else {
                                node.set_selected(false);
                            }
                        }
                    }


                    let edge_indices: Vec<_> = self.g.g().edge_indices().collect();


                    for e_idx in edge_indices {

                        if let Some((source, target)) = self.g.g().edge_endpoints(e_idx) {


                            if let Some(edge) = self.g.edge_mut(e_idx) {
                                if !selected_nodes.is_empty() &&
                                   (selected_nodes.contains(&source) || selected_nodes.contains(&target)) {
                                    edge.set_selected(true);
                                } else {
                                    edge.set_selected(false);
                                }
                            }
                        }
                    }
                }
            });

            ui.separator();

            let mut state = FruchtermanReingoldWithCenterGravityState::load(ui, None);

            if !self.physics_initialized {
                state.base.c_repulse = 20.0;
                state.base.c_attract = 0.001;
                state.base.k_scale = 20.0;
                state.base.dt = 1.0;

                self.physics_initialized = true;
            }

            ui.horizontal(|ui| {
                ui.label("Repulsion:");
                ui.add(egui::Slider::new(&mut state.base.c_repulse, 1.0..=500.0));

                ui.label("Atraction:");
                ui.add(egui::Slider::new(&mut state.base.c_attract, 0.0001..=0.1).logarithmic(true));
            });
            
            ui.horizontal(|ui| {
                ui.label("Scalar distance:");
                ui.add(egui::Slider::new(&mut state.base.k_scale, 1.0..=1000.0));

                ui.label("Velocity (dt):");
                ui.add(egui::Slider::new(&mut state.base.dt, 0.0001..=1.0).logarithmic(true));
            });

            state.save(ui, None);

            let style = SettingsStyle::default()
                .with_labels_always(true)
                .with_node_stroke_hook(|selected, dragged, node_color, stroke, egui_style| {
                    let mut s = stroke;
                    s.color = node_color.unwrap_or_else(|| 
                        egui_style.visuals.widgets.inactive.fg_stroke.color
                    );
                    if selected { s.width = 5.0; }
                    if dragged { s.color = egui::Color32::LIGHT_BLUE; }
                    s
                })
                .with_edge_stroke_hook(|selected, _order, stroke, _egui_style| {
                    let mut s = stroke; 

                    if selected {
                        s.color = Color32::RED;
                        s.width = 10.0;
                    }

                    s
                });
            
            let nav = SettingsNavigation::default()
                .with_fit_to_screen_enabled(self.auto_fit)
                .with_zoom_and_pan_enabled(true);

            ui.add(
                &mut GraphView::<
                    _, _, _, _, CustomNodeShape, DefaultEdgeShape,
                    FruchtermanReingoldWithCenterGravityState,
                    LayoutForceDirected<FruchtermanReingoldWithCenterGravity> 
                >::new(&mut self.g)
                .with_styles(&style)
                .with_navigations(&nav)
            );

            ui.ctx().request_repaint();
        });
    }
}

fn generate_graph(filter_input: &str) -> Result<(StableGraph<(), ()>, Vec<(NodeIndex, String)>), rusqlite::Error> {
    let mut g = StableGraph::new();
    let conn = Connection::open("kernel_graph.db")?;
    let mut id_map: HashMap<i32, NodeIndex> = HashMap::new();
    let mut labels = Vec::new();


    let mut cond_path = Vec::new();
    let mut cond_f_source = Vec::new();

    let filters: Vec<&str> = filter_input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();


    if filters.is_empty() {
        return Ok((g, labels));
    }


    for f in filters {
        cond_path.push(format!("path LIKE '{}%'", f));
        cond_f_source.push(format!("f_source.path LIKE '{}%'", f));
    }

    let where_path = cond_path.join(" OR ");
    let where_f_source = cond_f_source.join(" OR ");


    let query_files = format!("
        SELECT id, path FROM Files WHERE {}
        UNION
        SELECT f_target.id, f_target.path
        FROM Edges e
        JOIN Files f_source ON e.source_id = f_source.id
        JOIN Files f_target ON e.target_id = f_target.id
        WHERE {}
    ", where_path, where_f_source);

    let mut stmt_files = conn.prepare(&query_files)?;
    let file_iter = stmt_files.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    })?;

    for result in file_iter {
        let (id, path) = result?;
        let node_idx = g.add_node(()); 
        labels.push((node_idx, path));
        id_map.insert(id, node_idx);
    }


    let query_edges = format!("
        SELECT e.source_id, e.target_id
        FROM Edges e
        JOIN Files f_source ON e.source_id = f_source.id
        WHERE {}
    ", where_f_source);

    let mut stmt_edges = conn.prepare(&query_edges)?;
    let edge_iter = stmt_edges.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    })?;

    for result in edge_iter {
        let (source_id, target_id) = result?;
        

        if let (Some(&s_idx), Some(&t_idx)) = (id_map.get(&source_id), id_map.get(&target_id)) {
            g.add_edge(s_idx, t_idx, ());
        }
    }

    Ok((g, labels))
}

fn main() {
    run_native(
        "Vizualization in Graph Kernel",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(KernelGraphApp::new(cc)))),
    )
    .unwrap();
}
