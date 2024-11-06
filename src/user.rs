pub mod user {

    use uuid::Uuid;

    pub struct User {
        pub id: Uuid,
        sequence: String, //TODO: A representation should be possible with only 2 bits per char 
    }


    impl User {

        pub fn new() -> Self {
            User {
                id: Uuid::new_v4(),
                sequence: "".to_string(),
            }
        }
                
    }
}
