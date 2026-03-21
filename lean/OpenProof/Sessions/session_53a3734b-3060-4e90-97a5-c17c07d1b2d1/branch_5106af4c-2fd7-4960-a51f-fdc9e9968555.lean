import Mathlib
/-!
OpenProof scratch session: Coprime Divisor Cancels From a Product in `Nat`
Session id: session_53a3734b-3060-4e90-97a5-c17c07d1b2d1
Mode: research
Updated: 
-/
-- branch: Coprime Divisor Cancels From a Product in `Nat` planner [proving]
-- branch id: branch_5106af4c-2fd7-4960-a51f-fdc9e9968555
-- node: Coprime Divisor Cancels From a Product in `Nat` [proving]
-- kind: theorem
-- statement: theorem nat_coprime_dvd_of_dvd_mul_right {a b c : Nat} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c

-- artifact: Coprime Divisor Cancels From a Product in `Nat` planner [pending]
-- artifact id: artifact_branch_5106af4c-2fd7-4960-a51f-fdc9e9968555
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_5106af4c-2fd7-4960-a51f-fdc9e9968555
-- label: Coprime Divisor Cancels From a Product in `Nat` planner
-- metadata: {"hidden":true}

theorem nat_coprime_dvd_of_dvd_mul_right {a b c : Nat}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  have hdiv' : a ∣ c * b := by
    simpa [Nat.mul_comm] using hdiv
  exact hcop.dvd_of_dvd_mul_right hdiv'