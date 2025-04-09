use crate::ipc::protobuf::settings::application_settings::uisettings;
use popcorn_fx_core::core::media::Category;

impl From<&Category> for uisettings::Category {
    fn from(value: &Category) -> Self {
        match value {
            Category::Movies => uisettings::Category::MOVIES,
            Category::Series => uisettings::Category::SERIES,
            Category::Favorites => uisettings::Category::FAVORITES,
        }
    }
}
