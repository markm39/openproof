import Mathlib
/-!
OpenProof scratch session: Nat.Coprime Divides Cancellation on ℕ
Session id: session_527e5949-bc7c-481f-83ee-8aa556b1a49d
Mode: research
Updated: 
-/
-- branch: Nat.Coprime Divides Cancellation on ℕ planner [proving]
-- branch id: branch_6d2dcb4c-3d98-4ddf-a976-13525fd35ef1
-- node: Nat.Coprime Divides Cancellation on ℕ [proving]
-- kind: theorem
-- statement: theorem coprime_dvd_of_dvd_mul_right {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c

-- artifact: Nat.Coprime Divides Cancellation on ℕ planner [pending]
-- artifact id: artifact_branch_6d2dcb4c-3d98-4ddf-a976-13525fd35ef1
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_6d2dcb4c-3d98-4ddf-a976-13525fd35ef1
-- label: Nat.Coprime Divides Cancellation on ℕ planner
-- metadata: {"hidden":true}

theorem coprime_dvd_of_dvd_mul_right {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right hdiv

-- fallback candidate if the available lemma is the left-oriented sibling:
-- theorem coprime_dvd_of_dvd_mul_right {a b c : ℕ}
--     (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
--   have hdiv' : a ∣ c * b := by
--     simpa [Nat.mul_comm] using hdiv
--   exact hcop.dvd_of_dvd_mul_left hdiv'