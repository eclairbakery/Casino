use crate::services::shop::models::ShopItem;

pub fn get_shop_registry() -> Vec<ShopItem> {
    vec![
        ShopItem {
            id: 1,
            name: "miniVIP",
            description: "Taki mały VIP za bezcen. Przynajmniej można się flexować...",
            price: 5000_00,
            role_id: Some(1235550013233303582),
        },
        ShopItem {
            id: 2,
            name: "VIP",
            description: "No już porządna ranga na serwerze, która da Ci porządny flex i szacunek w kasynie.",
            price: 50_000_00,
            role_id: Some(1235548993933541397),
        },
        ShopItem {
            id: 3,
            name: "SVIP",
            description: "Ktoś tu lubi szaleć. Ktoś tu lubi flex. I to bardzo. Dlatego dostanie super VIPa (jak zasłuży)!",
            price: 150_000_00,
            role_id: Some(1235550115998076948),
        },
        ShopItem {
            id: 4,
            name: "MVIP",
            description: "Gość z ta rangą chyba poświęcił całe swoje życie na nudną ekonomię i przewala całą wypłatę na kasyno 💔",
            price: 5_000_000_00,
            role_id: Some(1235569694451306516),
        },
        ShopItem {
            id: 5,
            name: "Pieczywo VIP",
            description: "VIP final boss",
            price: 15_000_000_00,
            role_id: Some(1343632574437920799),
        },
    ]
}
