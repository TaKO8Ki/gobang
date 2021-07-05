use crate::app::App;
use tui::{backend::Backend, layout::Rect, Frame};

fn draw_database_list<B: Backend>(
    f: &mut Frame<'_, B>,
    app: &mut App,
    layout_chunk: Rect,
) -> anyhow::Result<()> {
    Ok(())
}
