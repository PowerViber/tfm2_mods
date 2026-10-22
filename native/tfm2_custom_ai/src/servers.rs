//! The host keeps one server extension per mod (`set_server_extension` replaces the previous one), so every
//! management-side service registers here and runs in a fixed order.
use mod_api_stable::{CommandResultV1, StableCommand, StableServerCtx, StableServerExtension};

pub struct Servers;
impl Servers {
    /// Order matters only for startup: objective HP settles career settings before profiles load.
    fn each(f: &mut dyn FnMut(&dyn StableServerExtension)) {
        f(&crate::objective_hp::ObjectiveRules);
        f(&crate::athlete_ai::ProfileServer);
    }
}
impl StableServerExtension for Servers {
    fn on_server_start(&self, ctx: &mut StableServerCtx<'_>) {
        crate::settings::reload();
        Self::each(&mut |s| s.on_server_start(ctx));
    }
    fn before_management_tick(&self, ctx: &mut StableServerCtx<'_>) {
        Self::each(&mut |s| s.before_management_tick(ctx));
    }
    fn after_management_tick(&self, ctx: &mut StableServerCtx<'_>) {
        Self::each(&mut |s| s.after_management_tick(ctx));
    }
    fn handle_command(&self, ctx: &mut StableServerCtx<'_>, command: &StableCommand<'_>) -> CommandResultV1 {
        let mut result = CommandResultV1::Pass;
        Self::each(&mut |s| {
            if result.code() == CommandResultV1::Pass.code() {
                result = s.handle_command(ctx, command);
            }
        });
        result
    }
}
