import Mathlib
/-!
OpenProof scratch session: Normalized Prime-gap Limit-point Conjecture
Session id: session_1f572764-1232-4551-9196-37744e622e65
Mode: research
Updated: 
-/
-- branch: Normalized Prime-gap Limit-point Conjecture repair [proving]
-- branch id: branch_6743b670-f423-4b33-8685-4fff7d98b961
-- node: Normalized Prime-gap Limit-point Conjecture [proving]
-- kind: theorem
-- statement: `∀ C : ℝ, 0 ≤ C → ∃ s : ℕ → ℕ, StrictMono s ∧ 2 ≤ s 0 ∧ Tendsto (fun i => ((p (s i + 1) - p (s i)) : ℝ) / Real.log (s i : ℝ)) atTop (𝓝 C)`, formalized relative to an abstract prime enumeration `p : ℕ → ℕ` packaged in a structure `PrimeEnumeration`.

-- artifact: Normalized Prime-gap Limit-point Conjecture repair [pending]
-- artifact id: artifact_branch_6743b670-f423-4b33-8685-4fff7d98b961
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_6743b670-f423-4b33-8685-4fff7d98b961
-- label: Normalized Prime-gap Limit-point Conjecture repair
-- metadata: {"hidden":true}

namespace PrimeEnumeration

lemma log_nat_ne_zero_of_two_le {n : ℕ} (hn : 2 ≤ n) :
    Real.log (n : ℝ) ≠ 0 := by
  have h1_nat : 1 < n := lt_of_lt_of_le (by decide : 1 < 2) hn
  have h1 : (1 : ℝ) < (n : ℝ) := by
    exact_mod_cast h1_nat
  exact ne_of_gt (Real.log_pos h1)

lemma log_prime_ne_zero (E : PrimeEnumeration) (n : ℕ) :
    Real.log (PrimeEnumeration.p E n : ℝ) ≠ 0 := by
  have htwo : 2 ≤ PrimeEnumeration.p E n := (PrimeEnumeration.prime_p E n).two_le
  have h1_nat : 1 < PrimeEnumeration.p E n := lt_of_lt_of_le (by decide : 1 < 2) htwo
  have h1 : (1 : ℝ) < (PrimeEnumeration.p E n : ℝ) := by
    exact_mod_cast h1_nat
  exact ne_of_gt (Real.log_pos h1)

lemma normalizedGapByIndex_eq_mul_logRatio (E : PrimeEnumeration) {n : ℕ} (hn : 2 ≤ n) :
    PrimeEnumeration.normalizedGapByIndex E n =
      PrimeEnumeration.normalizedGapByPrimeValue E n *
        (Real.log (PrimeEnumeration.p E n : ℝ) / Real.log (n : ℝ)) := by
  have hlogn : Real.log (n : ℝ) ≠ 0 :=
    PrimeEnumeration.log_nat_ne_zero_of_two_le hn
  have hlogp : Real.log (PrimeEnumeration.p E n : ℝ) ≠ 0 :=
    PrimeEnumeration.log_prime_ne_zero E n
  unfold PrimeEnumeration.normalizedGapByIndex PrimeEnumeration.normalizedGapByPrimeValue
  field_simp [hlogn, hlogp]
  ring

lemma factorization_by_log_ratio (E : PrimeEnumeration) {n : ℕ} (hn : 2 ≤ n) :
    PrimeEnumeration.normalizedGapByIndex E n =
      PrimeEnumeration.normalizedGapByPrimeValue E n *
        (Real.log (PrimeEnumeration.p E n : ℝ) / Real.log (n : ℝ)) :=
  PrimeEnumeration.normalizedGapByIndex_eq_mul_logRatio E hn

lemma zero_transfer_along_subseq (E : PrimeEnumeration) {s : ℕ → ℕ}
    (hs : PrimeEnumeration.AdmissibleSubseq s)
    (hgap :
      Filter.Tendsto
        (fun i => PrimeEnumeration.normalizedGapByPrimeValue E (s i))
        Filter.atTop (nhds 0))
    (hratio :
      Filter.Tendsto
        (fun i => Real.log (PrimeEnumeration.p E (s i) : ℝ) / Real.log (s i : ℝ))
        Filter.atTop (nhds 1)) :
    Filter.Tendsto
      (fun i => PrimeEnumeration.normalizedGapByIndex E (s i))
      Filter.atTop (nhds 0) := by
  have hsge : ∀ i, 2 ≤ s i := by
    intro i
    rcases hs with ⟨hmono, h0⟩
    exact le_trans h0 (hmono.monotone (Nat.zero_le i))
  have hmul :
      Filter.Tendsto
        (fun i =>
          PrimeEnumeration.normalizedGapByPrimeValue E (s i) *
            (Real.log (PrimeEnumeration.p E (s i) : ℝ) / Real.log (s i : ℝ)))
        Filter.atTop (nhds (0 * 1)) := by
    exact hgap.mul hratio
  have hEq :
      (fun i => PrimeEnumeration.normalizedGapByIndex E (s i)) =
        (fun i =>
          PrimeEnumeration.normalizedGapByPrimeValue E (s i) *
            (Real.log (PrimeEnumeration.p E (s i) : ℝ) / Real.log (s i : ℝ))) := by
    funext i
    exact PrimeEnumeration.normalizedGapByIndex_eq_mul_logRatio E (hsge i)
  simpa [hEq] using hmul

/-- Candidate strengthening: the log-comparison limit remains true along every admissible subsequence. -/
def LogComparisonSubseqCandidate (E : PrimeEnumeration) : Prop :=
  ∀ s : ℕ → ℕ, PrimeEnumeration.AdmissibleSubseq s →
    Filter.Tendsto
      (fun i => Real.log (PrimeEnumeration.p E (s i) : ℝ) / Real.log (s i : ℝ))
      Filter.atTop (nhds 1)

theorem zero_branch_transfer_of_subseq_log_comparison (E : PrimeEnumeration)
    (hsubseq : PrimeEnumeration.LogComparisonSubseqCandidate E) :
    PrimeEnumeration.ZeroBranchTransferCandidate E := by
  intro hzero
  change PrimeEnumeration.HasPrimeValueNormalizedLimitPointAt E 0 at hzero
  rcases hzero with ⟨s, hs, hgap⟩
  change PrimeEnumeration.HasLimitPointAt E 0
  refine ⟨s, hs, ?_⟩
  exact PrimeEnumeration.zero_transfer_along_subseq E hs hgap (hsubseq s hs)

end PrimeEnumeration