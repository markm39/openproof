import Mathlib
/-!
OpenProof scratch session: Normalized Prime-gap Subsequence Limit-point Conjecture
Session id: session_93d14fcf-aa49-4b48-a038-5272cbe94dfa
Mode: research
Updated: 
-/
-- branch: Normalized Prime-gap Subsequence Limit-point Conjecture prover [proving]
-- branch id: branch_13ac34bf-5318-4e3b-9e3a-eb5a2219d7f6
-- node: Normalized Prime-gap Subsequence Limit-point Conjecture [proving]
-- kind: theorem
-- statement: conjectural Lean proposition `PrimeGapSubseqLimitConjecture`, encoding that every `c ≥ 0` is a subsequential limit of the normalized prime-gap sequence `n ↦ ((p (n+1) - p n) / log n)` for an increasing prime enumeration `p`

-- artifact: Normalized Prime-gap Subsequence Limit-point Conjecture prover [pending]
-- artifact id: artifact_branch_13ac34bf-5318-4e3b-9e3a-eb5a2219d7f6
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_13ac34bf-5318-4e3b-9e3a-eb5a2219d7f6
-- label: Normalized Prime-gap Subsequence Limit-point Conjecture prover
-- metadata: {"hidden":false,"foreground":true}

def IsPrimeEnumeration (p : ℕ → ℕ) : Prop :=
  StrictMono p ∧ ∀ q : ℕ, Nat.Prime q ↔ ∃ n : ℕ, p n = q

def normalizedPrimeGap (p : ℕ → ℕ) (n : ℕ) : ℝ :=
  ((p (n + 1) - p n : ℕ) : ℝ) / Real.log (n : ℝ)

def HasPrimeGapSubseqLimit (p : ℕ → ℕ) (c : ℝ) : Prop :=
  ∃ s : ℕ → ℕ, StrictMono s ∧
    Filter.Tendsto (fun i : ℕ => normalizedPrimeGap p (s i)) Filter.atTop (nhds c)

def PrimeGapSubseqLimitConjectureAt (p : ℕ → ℕ) (c : ℝ) : Prop :=
  IsPrimeEnumeration p ∧ 0 ≤ c ∧ HasPrimeGapSubseqLimit p c

def PrimeGapSubseqLimitConjecture : Prop :=
  ∀ c : ℝ, 0 ≤ c →
    ∀ p : ℕ → ℕ, IsPrimeEnumeration p → HasPrimeGapSubseqLimit p c