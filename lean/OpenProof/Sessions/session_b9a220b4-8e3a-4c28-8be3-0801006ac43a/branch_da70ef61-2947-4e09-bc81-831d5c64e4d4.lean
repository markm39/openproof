import Mathlib
/-!
OpenProof scratch session: Coprime Divisor Cancels From a Product Over ℕ
Session id: session_b9a220b4-8e3a-4c28-8be3-0801006ac43a
Mode: research
Updated: 
-/
-- branch: Coprime Divisor Cancels From a Product Over ℕ planner [proving]
-- branch id: branch_da70ef61-2947-4e09-bc81-831d5c64e4d4
-- node: Coprime Divisor Cancels From a Product Over ℕ [proving]
-- kind: theorem
-- statement: `theorem nat_coprime_dvd_of_dvd_mul {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by`

-- artifact: Coprime Divisor Cancels From a Product Over ℕ planner [pending]
-- artifact id: artifact_branch_da70ef61-2947-4e09-bc81-831d5c64e4d4
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_da70ef61-2947-4e09-bc81-831d5c64e4d4
-- label: Coprime Divisor Cancels From a Product Over ℕ planner
-- metadata: {"hidden":true}

theorem nat_coprime_dvd_of_dvd_mul {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right (by simpa [Nat.mul_comm] using hdiv)