use mint::Vector2;


#[derive(Debug)]
pub enum UIControlEvent {
    MouseButtonDown {
        btn: UIMouseButton,
        position: Vector2<u32>,
    }
}

impl UIControlEvent {
    pub fn is_in_bound(&self, position: Vector2<u32>, size: Vector2<u32>) -> bool {
        match self {
            UIControlEvent::MouseButtonDown { position: event_position, .. } => {
                position.x <= event_position.x && event_position.x <= position.x + size.x &&
                position.y <= event_position.y && event_position.y <= position.y + size.y
            }
        }
    }
}

#[derive(Debug)]
pub enum UIMouseButton {
    Left,
    Rignt,
    Center,
}