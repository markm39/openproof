import Mathlib
/-!
OpenProof scratch session: Coprime Divisibility Through a Product on `Nat`
Session id: session_c3817bf8-baa1-4c97-9d00-487fedb9efb9
Mode: research
Updated: 
-/
-- branch: Coprime Divisibility Through a Product on `Nat` prover [proving]
-- branch id: branch_c8550d8f-e100-40d9-bb2b-16bae66ed0f0
-- node: Coprime Divisibility Through a Product on `Nat` [proving]
-- kind: theorem
-- statement: `theorem nat_coprime_dvd_of_dvd_mul_right {a b c : Nat} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Coprime Divisibility Through a Product on `Nat` prover [pending]
-- artifact id: artifact_branch_c8550d8f-e100-40d9-bb2b-16bae66ed0f0
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_c8550d8f-e100-40d9-bb2b-16bae66ed0f0
-- label: Coprime Divisibility Through a Product on `Nat` prover
-- metadata: {"hidden":false,"foreground":true}

theorem nat_coprime_dvd_of_dvd_mul_right {a b c : Nat}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right hdiv