use serde::Deserialize;

pub const ENDPOINT: &str = "https://api.hfs.purdue.edu/menus/v3/GraphQL";

pub const QUERY: &str = r#"
    query getFoodNames($date: Date!) {
        diningCourts {
            formalName
            dailyMenu(date: $date) {
                meals {
                    name
                    stations {
                        name
                        items {
                            item {
                                name
                            }
                        }
                    }
                }
            }
        }
    }
"#;

#[derive(Deserialize)]
pub struct Response {
    pub data: Data,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub dining_courts: Vec<DiningCourt>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiningCourt {
    pub formal_name: String,
    pub daily_menu: DailyMenu,
}

#[derive(Deserialize)]
pub struct DailyMenu {
    pub meals: Vec<Meal>,
}

#[derive(Deserialize)]
pub struct Meal {
    pub name: String,
    pub stations: Vec<Station>,
}

#[derive(Deserialize)]
pub struct Station {
    pub name: String,
    pub items: Vec<ItemShell>,
}

#[derive(Deserialize)]
pub struct ItemShell {
    pub item: Item,
}

#[derive(Deserialize)]
pub struct Item {
    pub name: String,
}
