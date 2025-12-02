// mod calc;
// mod help;
// mod start;
// mod stats;

// pub use calc::{
//     calc_handle, callback_handle, get_result, get_status, message_handle, run_result_watcher,
// };
// pub use help::help_handle;
// pub use start::{commands_help, start_handle};
// pub use stats::users_calc_handle;

// use serde::{Deserialize, Serialize};
// use teloxide::{
//     dispatching::dialogue::{Dialogue, InMemStorage},
//     utils::command::BotCommands,
// };

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub enum DialogueState {
//     #[default]
//     Idle,
//     AwaitingBaseIterations,
//     AwaitingFullCalc,
// }

// pub type BotDialogue = Dialogue<DialogueState, InMemStorage<DialogueState>>;

// #[derive(Debug, Clone, BotCommands)]
// #[command(rename_rule = "lowercase", description = "Доступные команды:")]
// pub enum Command {
//     /// Запуск и регистрация пользователя
//     Start,
//     /// Подсказка по командам
//     Help,
//     /// Меню расчетов
//     Calc,
//     /// Активные расчеты пользователей (админ)
//     UsersCalc,
// }
