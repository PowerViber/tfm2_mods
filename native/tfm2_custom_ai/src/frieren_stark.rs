// Included in the shared host-free Frieren core.
impl Core {
    fn cancel_leap(&mut self,out:&mut Vec<Cmd>){if self.leap.take().is_some(){out.push(Cmd::StarkCancel);}}
    fn update_stark(&mut self,input:&Input,out:&mut Vec<Cmd>){
        let t=input.tick;let m=input.owner;
        let Some(s)=input.stark.filter(|s|s.hp>0&&self.stark_requested&&t<self.stark_until) else {self.cancel_leap(out);return;};
        let guard=input.units.iter().find(|u|Some(u.id)==self.guard&&u.team==m.team&&u.hp>0).unwrap_or(m);
        let legal=|p:(i64,i64)| p.0>=0&&p.1>=0&&p.0<960_000&&p.1<960_000
            && !(input.wall)(p.0,p.1)&&distance(p,(guard.x,guard.y))<=(FR_STARK_LEASH as i128).pow(2);
        if self.leap.as_ref().is_some_and(|p|p.stark!=s.id||s.interrupted||!legal(p.to)) {self.cancel_leap(out);return;}
        if self.leap.is_none()&&self.leaps_enabled&&!s.interrupted&&t>=self.next_leap
            &&t+FR_STARK_LEAP_TICKS<self.stark_until {
            if let Some(u)=input.units.iter().filter(|u|u.team!=m.team&&u.hp>0&&u.visible&&u.targetable&&u.champion
                &&!self.jumped.contains(&u.id)&&legal((u.x,u.y))
                &&distance((s.x,s.y),(u.x,u.y))<=(FR_STARK_LEASH as i128).pow(2))
                .min_by_key(|u|(distance((s.x,s.y),(u.x,u.y)),u.id)) {
                self.jumped.push(u.id);self.next_leap=t+FR_STARK_LEAP_COOLDOWN_TICKS;
                self.leap=Some(Leap{stark:s.id,target:u.id,from:(s.x,s.y),to:(u.x,u.y),start:t,end:t+FR_STARK_LEAP_TICKS});
                out.push(Cmd::StarkWarning{x:u.x,y:u.y,ticks:FR_STARK_LEAP_TICKS});
            }
        }
        if let Some(p)=self.leap.clone(){
            let elapsed=t.saturating_sub(p.start).min(FR_STARK_LEAP_TICKS) as i64;
            let duration=FR_STARK_LEAP_TICKS as i64;
            let (x,y)=(p.from.0+(p.to.0-p.from.0)*elapsed/duration,p.from.1+(p.to.1-p.from.1)*elapsed/duration);
            out.push(Cmd::MoveStark{x,y});
            if t>=p.end {
                self.leap=None;out.push(Cmd::StarkCancel);out.push(Cmd::StarkLanding{x,y});
                let mut seen=std::collections::BTreeSet::new();
                for u in input.units.iter().filter(|u|u.team!=m.team&&u.hp>0&&u.visible&&u.targetable
                    &&distance((x,y),(u.x,u.y))<=(FR_STARK_LEAP_OUTER as i128).pow(2)) {
                    if !seen.insert(u.id){continue;}
                    let inner=distance((x,y),(u.x,u.y))<=(FR_STARK_LEAP_INNER as i128).pow(2);
                    if inner {let amount=FR_STARK_LEAP_BASE+m.ap*FR_STARK_LEAP_AP/100;out.push(Cmd::Damage{target:u.id,amount});self.requested_damage+=amount;}
                    out.push(Cmd::Control{target:u.id,airborne:if inner{FR_STARK_LEAP_AIR_TICKS}else{0},slow:FR_STARK_LEAP_SLOW,ticks:FR_STARK_LEAP_SLOW_TICKS});
                }
            }else if elapsed%2==0 {
                let height=4*FR_STARK_LEAP_HEIGHT*elapsed*(duration-elapsed)/(duration*duration);
                out.push(Cmd::StarkPose{x,y,height,descending:elapsed*2>=duration});
            }
            return; // No ordinary swing on the landing tick.
        }
        // Original walking/melee policy and clock, unchanged when leaps are disabled.
        let enemy=input.units.iter().filter(|u|u.team!=m.team&&u.champion&&u.hp>0&&u.visible&&u.targetable
            &&distance((u.x,u.y),(guard.x,guard.y))<=(FR_STARK_LEASH as i128).pow(2))
            .min_by_key(|u|(distance((u.x,u.y),(guard.x,guard.y)),u.id));
        let goal=enemy.map_or((guard.x,guard.y),|u|(u.x,u.y));
        if let Some(u)=enemy.filter(|u|distance((u.x,u.y),(s.x,s.y))<=(FR_STARK_REACH as i128).pow(2)){
            if t>=self.next_stark {let n=FR_STARK_BASE+m.ap*FR_STARK_AP/100;out.push(Cmd::Damage{target:u.id,amount:n});self.requested_damage+=n;self.next_stark=t+FR_STARK_INTERVAL_TICKS;}
        }else if distance((s.x,s.y),goal)>12_000i128.pow(2){
            let (dx,dy)=direction(goal.0-s.x,goal.1-s.y);let next=(s.x+dx/10,s.y+dy/10);
            if !(input.wall)(next.0,next.1){out.push(Cmd::MoveStark{x:next.0,y:next.1});}
        }
    }
}
