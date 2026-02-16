pub struct ShopItem {
    pub id: i32,
    pub name: &'static str,
    pub description: &'static str,
    pub price: i64,
    pub role_id: Option<u64>,
}
