import Mathlib
/-!
OpenProof scratch session: Coprime Divisibility Through a Product on `Nat`
Session id: session_c3817bf8-baa1-4c97-9d00-487fedb9efb9
Mode: research
Updated: 
-/
-- branch: Coprime Divisibility Through a Product on `Nat` planner [proving]
-- branch id: branch_a5f34b55-f83c-4c16-afe1-0d4eeea587d3
-- node: Coprime Divisibility Through a Product on `Nat` [proving]
-- kind: theorem
-- statement: `theorem nat_coprime_dvd_of_dvd_mul_right {a b c : Nat} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Coprime Divisibility Through a Product on `Nat` planner [pending]
-- artifact id: artifact_branch_a5f34b55-f83c-4c16-afe1-0d4eeea587d3
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_a5f34b55-f83c-4c16-afe1-0d4eeea587d3
-- label: Coprime Divisibility Through a Product on `Nat` planner
-- metadata: {"hidden":true}

theorem nat_coprime_dvd_of_dvd_mul_right {a b c : Nat}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  have hdiv' : a ∣ c * b := by
    simpa [Nat.mul_comm] using hdiv
  exact hcop.dvd_of_dvd_mul_right hdiv'