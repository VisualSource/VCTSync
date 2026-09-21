use crate::host::Host;
use crate::render::render_tree;
use crate::runtime::Event;
use iced::Element;

pub fn surface<'a>(rt: &'a Host, id: &str) -> Element<'a, Event> {
    let Some(tree) = rt.trees.get(id) else {
        return iced::widget::space().into();
    };

    render_tree(tree)
}
