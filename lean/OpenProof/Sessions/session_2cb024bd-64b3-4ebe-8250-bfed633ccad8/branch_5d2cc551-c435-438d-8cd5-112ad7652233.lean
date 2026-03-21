import Mathlib
/-!
OpenProof scratch session: Euclid Lemma For Naturals
Session id: session_2cb024bd-64b3-4ebe-8250-bfed633ccad8
Mode: research
Updated: 
-/
-- branch: Euclid Lemma For Naturals repair [proving]
-- branch id: branch_5d2cc551-c435-438d-8cd5-112ad7652233
-- node: Euclid Lemma For Naturals [proving]
-- kind: theorem
-- statement: `theorem nat_dvd_of_coprime_dvd_mul {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Euclid Lemma For Naturals repair [pending]
-- artifact id: artifact_branch_5d2cc551-c435-438d-8cd5-112ad7652233
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_5d2cc551-c435-438d-8cd5-112ad7652233
-- label: Euclid Lemma For Naturals repair
-- metadata: {"hidden":true}

theorem nat_dvd_of_coprime_dvd_mul {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  have hdiv' : a ∣ c * b := by
    simpa [Nat.mul_comm] using hdiv
  exact hcop.dvd_of_dvd_mul_right hdiv'