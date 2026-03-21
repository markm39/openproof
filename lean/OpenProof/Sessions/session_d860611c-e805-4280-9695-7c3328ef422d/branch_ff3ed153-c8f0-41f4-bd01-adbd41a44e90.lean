import Mathlib
/-!
OpenProof scratch session: Coprime Cancellation in a Divisibility Product
Session id: session_d860611c-e805-4280-9695-7c3328ef422d
Mode: research
Updated: 
-/
-- branch: Coprime Cancellation in a Divisibility Product repair [proving]
-- branch id: branch_ff3ed153-c8f0-41f4-bd01-adbd41a44e90
-- node: Coprime Cancellation in a Divisibility Product [proving]
-- kind: theorem
-- statement: theorem coprime_dvd_of_dvd_mul {a b c : Nat} (hcop : Nat.Coprime a b) (h : a ∣ b * c) : a ∣ c

-- artifact: Coprime Cancellation in a Divisibility Product repair [pending]
-- artifact id: artifact_branch_ff3ed153-c8f0-41f4-bd01-adbd41a44e90
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_ff3ed153-c8f0-41f4-bd01-adbd41a44e90
-- label: Coprime Cancellation in a Divisibility Product repair
-- metadata: {"hidden":true}

theorem coprime_dvd_of_dvd_mul {a b c : Nat}
    (hcop : Nat.Coprime a b) (h : a ∣ b * c) : a ∣ c := by
  have h' : a ∣ c * b := by
    simpa [Nat.mul_comm] using h
  exact hcop.dvd_of_dvd_mul_right h'