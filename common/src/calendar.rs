use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Calendar
{
    /// If this is a nil UUID (it's a thing, look it up)
    /// it means the calendar does not exist in the database.
    /// This is useful when deserializing Calendar
    /// for create requests.
    #[serde(default = "Uuid::nil")]
    id: Uuid,
}


impl Calendar
{
    pub fn new(id: Uuid) -> Calendar
    {
        Calendar {
            id
        }
    }

    pub fn get_id(&self) -> Uuid { self.id }
}