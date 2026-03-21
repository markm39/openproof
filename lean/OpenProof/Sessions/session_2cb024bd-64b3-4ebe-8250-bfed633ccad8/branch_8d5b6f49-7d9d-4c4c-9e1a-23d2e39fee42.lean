import Mathlib
/-!
OpenProof scratch session: Euclid Lemma For Naturals
Session id: session_2cb024bd-64b3-4ebe-8250-bfed633ccad8
Mode: research
Updated: 
-/
-- branch: Euclid Lemma For Naturals planner [proving]
-- branch id: branch_8d5b6f49-7d9d-4c4c-9e1a-23d2e39fee42
-- node: Euclid Lemma For Naturals [proving]
-- kind: theorem
-- statement: `theorem nat_dvd_of_coprime_dvd_mul {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Euclid Lemma For Naturals planner [pending]
-- artifact id: artifact_branch_8d5b6f49-7d9d-4c4c-9e1a-23d2e39fee42
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_8d5b6f49-7d9d-4c4c-9e1a-23d2e39fee42
-- label: Euclid Lemma For Naturals planner
-- metadata: {"hidden":true}

theorem nat_dvd_of_coprime_dvd_mul {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right hdiv

-- Fallback if only the other-handed lemma elaborates:
theorem nat_dvd_of_coprime_dvd_mul_alt {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  have hdiv' : a ∣ c * b := by
    simpa [Nat.mul_comm] using hdiv
  exact hcop.symm.dvd_of_dvd_mul_left hdiv'