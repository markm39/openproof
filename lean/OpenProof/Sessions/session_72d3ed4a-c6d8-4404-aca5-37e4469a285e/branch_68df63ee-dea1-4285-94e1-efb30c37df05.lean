import Mathlib
/-!
OpenProof scratch session: Coprime Divisor Cancels From a Product in ℕ
Session id: session_72d3ed4a-c6d8-4404-aca5-37e4469a285e
Mode: research
Updated: 
-/
-- branch: Coprime Divisor Cancels From a Product in ℕ planner [proving]
-- branch id: branch_68df63ee-dea1-4285-94e1-efb30c37df05
-- node: Coprime Divisor Cancels From a Product in ℕ [proving]
-- kind: theorem
-- statement: `theorem coprime_dvd_of_dvd_mul_right {a b c : Nat} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Coprime Divisor Cancels From a Product in ℕ planner [pending]
-- artifact id: artifact_branch_68df63ee-dea1-4285-94e1-efb30c37df05
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_68df63ee-dea1-4285-94e1-efb30c37df05
-- label: Coprime Divisor Cancels From a Product in ℕ planner
-- metadata: {"hidden":true}

theorem coprime_dvd_of_dvd_mul_right {a b c : Nat}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right hdiv

-- possible fallback if the available lemma is oriented differently:
-- theorem coprime_dvd_of_dvd_mul_right {a b c : Nat}
--     (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
--   simpa [Nat.mul_comm] using hcop.dvd_of_dvd_mul_left (by simpa [Nat.mul_comm] using hdiv)