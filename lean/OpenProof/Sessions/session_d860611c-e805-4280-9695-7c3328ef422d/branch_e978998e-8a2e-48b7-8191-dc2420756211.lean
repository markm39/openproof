import Mathlib
/-!
OpenProof scratch session: Coprime Cancellation in a Divisibility Product
Session id: session_d860611c-e805-4280-9695-7c3328ef422d
Mode: research
Updated: 
-/
-- branch: Coprime Cancellation in a Divisibility Product planner [proving]
-- branch id: branch_e978998e-8a2e-48b7-8191-dc2420756211
-- node: Coprime Cancellation in a Divisibility Product [proving]
-- kind: theorem
-- statement: theorem coprime_dvd_of_dvd_mul {a b c : Nat} (hcop : Nat.Coprime a b) (h : a ∣ b * c) : a ∣ c

-- artifact: Coprime Cancellation in a Divisibility Product planner [pending]
-- artifact id: artifact_branch_e978998e-8a2e-48b7-8191-dc2420756211
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_e978998e-8a2e-48b7-8191-dc2420756211
-- label: Coprime Cancellation in a Divisibility Product planner
-- metadata: {"hidden":true}

theorem coprime_dvd_of_dvd_mul {a b c : Nat}
    (hcop : Nat.Coprime a b) (h : a ∣ b * c) : a ∣ c := by
  simpa [Nat.mul_comm] using hcop.dvd_of_dvd_mul_right h