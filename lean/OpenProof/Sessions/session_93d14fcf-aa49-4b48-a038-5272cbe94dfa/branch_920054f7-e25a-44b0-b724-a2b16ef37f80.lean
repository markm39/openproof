import Mathlib
/-!
OpenProof scratch session: Normalized Prime-gap Subsequence Limit-point Conjecture
Session id: session_93d14fcf-aa49-4b48-a038-5272cbe94dfa
Mode: research
Updated: 
-/
-- branch: Normalized Prime-gap Subsequence Limit-point Conjecture planner [proving]
-- branch id: branch_920054f7-e25a-44b0-b724-a2b16ef37f80
-- node: Normalized Prime-gap Subsequence Limit-point Conjecture [verifying]
-- kind: theorem
-- statement: conjectural Lean proposition `PrimeGapSubseqLimitConjecture`, encoding that every `c ≥ 0` is a subsequential limit of the normalized prime-gap sequence `n ↦ ((p (n+1) - p n) / log n)` for an increasing prime enumeration `p`

-- artifact: Normalized Prime-gap Subsequence Limit-point Conjecture planner [pending]
-- artifact id: artifact_branch_920054f7-e25a-44b0-b724-a2b16ef37f80
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_920054f7-e25a-44b0-b724-a2b16ef37f80
-- label: Normalized Prime-gap Subsequence Limit-point Conjecture planner
-- metadata: {"hidden":true}

noncomputable section

def IsPrimeEnumeration (p : ℕ → ℕ) : Prop :=
  StrictMono p ∧ ∀ q : ℕ, Nat.Prime q ↔ ∃ n : ℕ, p n = q

def normalizedPrimeGap (p : ℕ → ℕ) (n : ℕ) : ℝ :=
  ((p (n + 1) - p n : ℕ) : ℝ) / Real.log (n : ℝ)

def HasPrimeGapSubseqLimit (p : ℕ → ℕ) (c : ℝ) : Prop :=
  ∃ s : ℕ → ℕ, StrictMono s ∧
    Filter.Tendsto (fun i : ℕ => normalizedPrimeGap p (s i)) Filter.atTop (nhds c)

def HasPrimeGapSubseqLimitFromTwo (p : ℕ → ℕ) (c : ℝ) : Prop :=
  ∃ s : ℕ → ℕ, StrictMono s ∧
    Filter.Tendsto (fun i : ℕ => normalizedPrimeGap p (s i + 2)) Filter.atTop (nhds c)

def PrimeGapSubseqLimitConjecture : Prop :=
  ∀ c : ℝ, 0 ≤ c →
    ∀ p : ℕ → ℕ, IsPrimeEnumeration p → HasPrimeGapSubseqLimit p c

def PrimeGapSubseqLimitConjectureShifted : Prop :=
  ∀ c : ℝ, 0 ≤ c →
    ∀ p : ℕ → ℕ, IsPrimeEnumeration p → HasPrimeGapSubseqLimitFromTwo p c

lemma IsPrimeEnumeration.strictMono {p : ℕ → ℕ} (hp : IsPrimeEnumeration p) : StrictMono p :=
  hp.1

lemma prime_at {p : ℕ → ℕ} (hp : IsPrimeEnumeration p) (n : ℕ) : Nat.Prime (p n) := by
  exact (hp.2 (p n)).2 ⟨n, rfl⟩

lemma prime_gap_num_pos {p : ℕ → ℕ} (hp : IsPrimeEnumeration p) (n : ℕ) :
    0 < p (n + 1) - p n := by
  exact Nat.sub_pos_of_lt (hp.strictMono (Nat.lt_succ_self n))

lemma prime_two_le {p : ℕ → ℕ} (hp : IsPrimeEnumeration p) (n : ℕ) : 2 ≤ p n := by
  exact (prime_at hp n).two_le

lemma log_nat_add_two_pos (n : ℕ) : 0 < Real.log (n + 2 : ℝ) := by
  have h : (1 : ℝ) < (n + 2 : ℝ) := by
    exact_mod_cast (show 1 < n + 2 by omega)
  exact Real.log_pos h

lemma HasPrimeGapSubseqLimitFromTwo.to_unshifted {p : ℕ → ℕ} {c : ℝ} :
    HasPrimeGapSubseqLimitFromTwo p c → HasPrimeGapSubseqLimit p c := by
  rintro ⟨s, hs, hlim⟩
  refine ⟨fun i => s i + 2, ?_, ?_⟩
  · intro a b hab
    exact Nat.add_lt_add_right (hs hab) 2
  · simpa using hlim

lemma PrimeGapSubseqLimitConjectureShifted.implies :
    PrimeGapSubseqLimitConjectureShifted → PrimeGapSubseqLimitConjecture := by
  intro h c hc p hp
  exact HasPrimeGapSubseqLimitFromTwo.to_unshifted (h c hc p hp)

end