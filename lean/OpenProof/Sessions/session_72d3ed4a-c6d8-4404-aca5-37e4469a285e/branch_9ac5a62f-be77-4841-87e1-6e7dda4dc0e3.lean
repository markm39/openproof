import Mathlib
/-!
OpenProof scratch session: Coprime Divisor Cancels From a Product in ℕ
Session id: session_72d3ed4a-c6d8-4404-aca5-37e4469a285e
Mode: research
Updated: 
-/
-- branch: Coprime Divisor Cancels From a Product in ℕ repair [proving]
-- branch id: branch_9ac5a62f-be77-4841-87e1-6e7dda4dc0e3
-- node: Coprime Divisor Cancels From a Product in ℕ [proving]
-- kind: theorem
-- statement: `theorem coprime_dvd_of_dvd_mul_right {a b c : Nat} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Coprime Divisor Cancels From a Product in ℕ repair [pending]
-- artifact id: artifact_branch_9ac5a62f-be77-4841-87e1-6e7dda4dc0e3
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_9ac5a62f-be77-4841-87e1-6e7dda4dc0e3
-- label: Coprime Divisor Cancels From a Product in ℕ repair
-- metadata: {"hidden":true}

theorem coprime_dvd_of_dvd_mul_right {a b c : Nat}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right (by
    simpa [Nat.mul_comm] using hdiv)