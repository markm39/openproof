import Mathlib
/-!
OpenProof scratch session: Normalized Prime-gap Limit-point Conjecture
Session id: session_1f572764-1232-4551-9196-37744e622e65
Mode: research
Updated: 
-/
-- branch: Normalized Prime-gap Limit-point Conjecture planner [proving]
-- branch id: branch_5734c552-f171-41b4-9ca3-dac4ea925a44
-- node: Normalized Prime-gap Limit-point Conjecture [proving]
-- kind: theorem
-- statement: `∀ C : ℝ, 0 ≤ C → ∃ s : ℕ → ℕ, StrictMono s ∧ 2 ≤ s 0 ∧ Tendsto (fun i => ((p (s i + 1) - p (s i)) : ℝ) / Real.log (s i : ℝ)) atTop (𝓝 C)`, formalized relative to an abstract prime enumeration `p : ℕ → ℕ` packaged in a structure `PrimeEnumeration`.

-- artifact: Normalized Prime-gap Limit-point Conjecture planner [pending]
-- artifact id: artifact_branch_5734c552-f171-41b4-9ca3-dac4ea925a44
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_5734c552-f171-41b4-9ca3-dac4ea925a44
-- label: Normalized Prime-gap Limit-point Conjecture planner
-- metadata: {"hidden":true}

namespace PrimeEnumeration

lemma log_nat_ne_zero_of_two_le {n : ℕ} (hn : 2 ≤ n) :
    Real.log (n : ℝ) ≠ 0 := by
  have h1_nat : 1 < n := lt_of_lt_of_le (by decide : 1 < 2) hn
  have h1 : (1 : ℝ) < (n : ℝ) := by
    exact_mod_cast h1_nat
  exact ne_of_gt (Real.log_pos h1)

lemma log_prime_ne_zero (E : PrimeEnumeration) (n : ℕ) :
    Real.log (E.p n : ℝ) ≠ 0 := by
  have h1_nat : 1 < E.p n := lt_of_lt_of_le (by decide : 1 < 2) (E.prime_p n).two_le
  have h1 : (1 : ℝ) < (E.p n : ℝ) := by
    exact_mod_cast h1_nat
  exact ne_of_gt (Real.log_pos h1)

lemma normalizedGapByIndex_eq_mul_logRatio (E : PrimeEnumeration) {n : ℕ} (hn : 2 ≤ n) :
    E.normalizedGapByIndex n =
      E.normalizedGapByPrimeValue n *
        (Real.log (E.p n : ℝ) / Real.log (n : ℝ)) := by
  have hlogn : Real.log (n : ℝ) ≠ 0 := log_nat_ne_zero_of_two_le hn
  have hlogp : Real.log (E.p n : ℝ) ≠ 0 := E.log_prime_ne_zero n
  unfold normalizedGapByIndex normalizedGapByPrimeValue
  field_simp [hlogn, hlogp]
  ring

lemma zero_transfer_along_subseq (E : PrimeEnumeration) {s : ℕ → ℕ}
    (hs : AdmissibleSubseq s)
    (hgap : Tendsto (fun i => E.normalizedGapByPrimeValue (s i)) atTop (𝓝 0))
    (hratio : Tendsto (fun i => Real.log (E.p (s i) : ℝ) / Real.log (s i : ℝ)) atTop (𝓝 1)) :
    Tendsto (fun i => E.normalizedGapByIndex (s i)) atTop (𝓝 0) := by
  have hEq :
      (fun i => E.normalizedGapByIndex (s i)) =
        (fun i =>
          E.normalizedGapByPrimeValue (s i) *
            (Real.log (E.p (s i) : ℝ) / Real.log (s i : ℝ))) := by
    funext i
    exact E.normalizedGapByIndex_eq_mul_logRatio (two_le_of_admissible hs i)
  rw [hEq]
  simpa using hgap.mul hratio

/-- Candidate lifting of the global `log p_n / log n → 1` comparison to admissible subsequences. -/
def LogComparisonAlongAdmissibleSubseqCandidate (E : PrimeEnumeration) : Prop :=
  ∀ ⦃s : ℕ → ℕ⦄, AdmissibleSubseq s →
    LogComparisonCandidate E →
    Tendsto (fun i => Real.log (E.p (s i) : ℝ) / Real.log (s i : ℝ)) atTop (𝓝 1)

/-- Planned zero-branch reduction through the standard `log p_n` normalization. -/
def ZeroBranchViaPrimeValuePlan (E : PrimeEnumeration) : Prop :=
  ZeroBranchByPrimeValueStatement E →
    LogComparisonAlongAdmissibleSubseqCandidate E →
    ZeroBranchStatement E

end PrimeEnumeration