use serenity::builder::CreateCommand;

pub fn register() -> CreateCommand {
    CreateCommand::new("show").description("Your list~? Leave it to Rengo-chan to show you nicely!")
}
