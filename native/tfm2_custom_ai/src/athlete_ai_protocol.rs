//! Tactical-lab adapter. Reuses the native decision, cooldown and shield rules.
use crate::athlete_ai_core::*;
#[derive(Default)]
pub struct Lab {
    clock: TeleportClock,
    shield: Option<ShieldLedger>,
}
impl Lab {
    pub fn run(&mut self, a: &[i64], command: u32) -> String {
        let Some(f) = crate::gojo_v2_protocol::Frame::decode(a) else {
            return "{\"error\":\"Invalid tactical frame\"}".into();
        };
        let Some(me) = f.scene.actors.iter().find(|u| u.id == f.owner) else {
            return "{\"error\":\"Missing owner\"}".into();
        };
        let mut recipients = vec![];
        let mut accepted = false;
        let mut reason = "Evaluate visible information";
        match command {
            1 if f.unlocked[1] && self.clock.para_ready(f.tick) => {
                self.clock.commit_para(f.tick, f.haste);
                accepted = true;
                reason = "Paranoia accepted; Step remains independent"
            }
            2 if f.unlocked[1] && self.clock.step_ready(f.tick) => {
                self.clock.commit_step(f.tick, f.haste);
                accepted = true;
                reason = "Step accepted; destination fixed for 36 ticks"
            }
            3 if f.unlocked[2] && self.clock.ult_ready(f.tick) => {
                self.clock.commit_ult(f.tick, f.ult_haste);
                accepted = true;
                reason = "Ultimate accepted; destination fixed for 120 ticks"
            }
            4 if f
                .scene
                .actors
                .iter()
                .any(|u| u.id == f.selected && u.team == me.team && u.hp > 0) =>
            {
                self.shield = Some(ShieldLedger::new(f.selected));
                recipients.push(f.selected);
                accepted = true;
                reason = "Cast-time shield; selected ally entered in ledger"
            }
            5 => {
                if let Some(ledger) = self.shield.as_mut() {
                    recipients = ledger.first_marker(
                        me.team,
                        f.aim,
                        &f.scene
                            .actors
                            .iter()
                            .map(|u| (u.id, u.team, u.hp, u.point))
                            .collect::<Vec<_>>(),
                    );
                    accepted = true;
                    reason = "First-marker snapshot; no refresh or late entrants"
                }
            }
            6 => {
                self.shield = None;
                accepted = true;
                reason = "Cast cancelled; existing shields expire normally"
            }
            0 => {}
            _ => reason = "Rejected: locked slot, cooldown, or no active shield cast",
        }
        let plans=[("Dio",76000,2,120),("Full Void",64000,60,240),("Quick Void",96000,12,60)].iter().map(|(name,r,opening,control)|{
   let p=f.scene.engage(f.profile,f.owner,me.point,*r,*opening,*control,false,0);
   p.map_or_else(||format!("{{\"name\":{name:?},\"reason\":\"Wait: no visible eligible enemy\",\"score\":null}}"),|p|format!("{{\"name\":{name:?},\"score\":{},\"guest\":{},\"caught\":{},\"incoming\":{},\"alliedLossTicks\":{},\"reason\":{:?}}}",p.score,p.guest.map_or(-1,|v|v as i64),p.caught,p.incoming,p.allied_loss_ticks,p.reason))
  }).collect::<Vec<_>>().join(",");
        let teleport = f
            .scene
            .teleport_score(f.profile, f.owner, f.aim, 42000, 120, 0);
        format!("{{\"version\":1,\"profileVersion\":{PROFILE_VERSION},\"profileKnown\":{},\"accepted\":{accepted},\"reason\":{reason:?},\"plans\":[{plans}],\"teleportScore\":{teleport},\"cooldowns\":[{},{},{}],\"shieldAmount\":{AEGIS_SHIELD},\"shieldTicks\":{AEGIS_SHIELD_TICKS},\"shieldRecipients\":{recipients:?},\"served\":{:?}}}",f.profile.known,self.clock.para,self.clock.step,self.clock.ultimate,self.shield.as_ref().map(|s|s.served.clone()).unwrap_or_default())
    }
}
