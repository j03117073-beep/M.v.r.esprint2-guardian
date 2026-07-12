theory Kernel_Verification
  imports Main
begin

(*
  Formal model of the kernel control layer described in src/kernel.rs.
  The model covers:
    - authority modes: PassThrough, Clamp, RateLimit, FallbackToDroop
    - runtime states: Normal, Degraded, Incoherent, Emergency
    - guard conditions for admissible commands and events
*)

datatype authority = PassThrough | Clamp | RateLimit | FallbackToDroop
datatype kernel_state = Normal | Degraded | Incoherent | Emergency
datatype event = NoViolation | ThermalViolation | FrequencyViolation | CommunicationViolation

fun next_state :: "kernel_state => authority => kernel_state" where
  "next_state Normal PassThrough = Normal" |
  "next_state Normal Clamp = Degraded" |
  "next_state Normal RateLimit = Degraded" |
  "next_state Normal FallbackToDroop = Emergency" |
  "next_state Degraded PassThrough = Degraded" |
  "next_state Degraded Clamp = Degraded" |
  "next_state Degraded RateLimit = Degraded" |
  "next_state Degraded FallbackToDroop = Emergency" |
  "next_state Incoherent _ = Emergency" |
  "next_state Emergency _ = Emergency"

fun authority_is_safe :: "authority => bool" where
  "authority_is_safe PassThrough = True" |
  "authority_is_safe RateLimit = True" |
  "authority_is_safe Clamp = False" |
  "authority_is_safe FallbackToDroop = False"

fun event_is_safe :: "event => bool" where
  "event_is_safe NoViolation = True" |
  "event_is_safe ThermalViolation = False" |
  "event_is_safe FrequencyViolation = False" |
  "event_is_safe CommunicationViolation = False"

fun guarded_step :: "kernel_state => authority => event => kernel_state" where
  "guarded_step s a e =
     (if authority_is_safe a then
        (if event_is_safe e then next_state s a else Emergency)
      else Emergency)"

lemma normal_safe_step_stays_non_emergency:
  fixes s :: kernel_state and a :: authority and e :: event
  assumes "s = Normal" and "a = PassThrough" and "event_is_safe e"
  shows "guarded_step s a e = Normal"
proof (cases e)
  case NoViolation
  then show ?thesis using assms by simp
next
  case ThermalViolation
  then show ?thesis using assms by simp
next
  case FrequencyViolation
  then show ?thesis using assms by simp
next
  case CommunicationViolation
  then show ?thesis using assms by simp
qed

lemma degraded_safe_step_stays_non_emergency:
  fixes s :: kernel_state and a :: authority and e :: event
  assumes "s = Degraded" and "a = PassThrough" and "event_is_safe e"
  shows "guarded_step s a e = Degraded"
proof (cases e)
  case NoViolation
  then show ?thesis using assms by simp
next
  case ThermalViolation
  then show ?thesis using assms by simp
next
  case FrequencyViolation
  then show ?thesis using assms by simp
next
  case CommunicationViolation
  then show ?thesis using assms by simp
qed

end
