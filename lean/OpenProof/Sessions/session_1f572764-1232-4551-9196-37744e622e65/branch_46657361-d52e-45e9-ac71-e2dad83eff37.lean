import Mathlib
/-!
OpenProof scratch session: Normalized Prime-gap Limit-point Conjecture
Session id: session_1f572764-1232-4551-9196-37744e622e65
Mode: research
Updated: 
-/
-- branch: Normalized Prime-gap Limit-point Conjecture prover [proving]
-- branch id: branch_46657361-d52e-45e9-ac71-e2dad83eff37
-- node: Normalized Prime-gap Limit-point Conjecture [proving]
-- kind: theorem
-- statement: `∀ C : ℝ, 0 ≤ C → ∃ s : ℕ → ℕ, StrictMono s ∧ 2 ≤ s 0 ∧ Tendsto (fun i => ((p (s i + 1) - p (s i)) : ℝ) / Real.log (s i : ℝ)) atTop (𝓝 C)`, formalized relative to an abstract increasing exhaustive prime enumeration `p`.

-- artifact: Normalized Prime-gap Limit-point Conjecture prover [pending]
-- artifact id: artifact_branch_46657361-d52e-45e9-ac71-e2dad83eff37
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_46657361-d52e-45e9-ac71-e2dad83eff37
-- label: Normalized Prime-gap Limit-point Conjecture prover
-- metadata: {"hidden":false,"foreground":true}

import Mathlib

open Filter
open scoped Topology

noncomputable section

/-- An increasing exhaustive enumeration of the prime numbers. -/
structure PrimeEnumeration where
  p : ℕ → ℕ
  strictMono_p : StrictMono p
  prime_p : ∀ n, Nat.Prime (p n)
  exhaustive_p : ∀ q, Nat.Prime q → ∃ n, p n = q

namespace PrimeEnumeration

def gap (E : PrimeEnumeration) (n : ℕ) : ℕ :=
  E.p (n + 1) - E.p n

/-- The literal normalization requested by the user: divide by `log n`. -/
def normalizedGapByIndex (E : PrimeEnumeration) (n : ℕ) : ℝ :=
  (E.gap n : ℝ) / Real.log (n : ℝ)

/-- The standard normalization appearing in the prime-gap literature: divide by `log p_n`. -/
def normalizedGapByPrimeValue (E : PrimeEnumeration) (n : ℕ) : ℝ :=
  (E.gap n : ℝ) / Real.log (E.p n : ℝ)

def AdmissibleSubseq (s : ℕ → ℕ) : Prop :=
  StrictMono s ∧ 2 ≤ s 0

def HasLimitPointAt (E : PrimeEnumeration) (C : ℝ) : Prop :=
  ∃ s : ℕ → ℕ, AdmissibleSubseq s ∧
    Tendsto (fun i => E.normalizedGapByIndex (s i)) atTop (𝓝 C)

def HasPrimeValueNormalizedLimitPointAt (E : PrimeEnumeration) (C : ℝ) : Prop :=
  ∃ s : ℕ → ℕ, AdmissibleSubseq s ∧
    Tendsto (fun i => E.normalizedGapByPrimeValue (s i)) atTop (𝓝 C)

/-- Main conjecture matching the user's statement. -/
def NormalizedPrimeGapLimitPointConjecture (E : PrimeEnumeration) : Prop :=
  ∀ ⦃C : ℝ⦄, 0 ≤ C → E.HasLimitPointAt C

/-- Encodes the `C = 0` branch in the user's normalization. -/
def ZeroBranchStatement (E : PrimeEnumeration) : Prop :=
  E.HasLimitPointAt 0

/-- Encodes the standard `C = 0` branch in the `log p_n` normalization. -/
def ZeroBranchByPrimeValueStatement (E : PrimeEnumeration) : Prop :=
  E.HasPrimeValueNormalizedLimitPointAt 0

/-- “For every `ε > 0`, there are arbitrarily large indices with normalized gap `< ε`.” -/
def ArbitrarilySmallNormalizedGaps (E : PrimeEnumeration) : Prop :=
  ∀ ε > 0, ∀ N : ℕ, ∃ n ≥ N, E.normalizedGapByIndex n < ε

/-- Candidate reduction: arbitrarily small values should yield a subsequence converging to `0`. -/
def ZeroBranchReductionCandidate (E : PrimeEnumeration) : Prop :=
  ArbitrarilySmallNormalizedGaps E → ZeroBranchStatement E

/-- Candidate PNT-style comparison needed to transfer `log p_n` results to `log n`. -/
def LogComparisonCandidate (E : PrimeEnumeration) : Prop :=
  Tendsto
    (fun n => Real.log (E.p (n + 2) : ℝ) / Real.log ((n + 2 : ℕ) : ℝ))
    atTop (𝓝 1)

/-- Candidate transfer from the standard normalization to the user's normalization at `C = 0`. -/
def ZeroBranchTransferCandidate (E : PrimeEnumeration) : Prop :=
  ZeroBranchByPrimeValueStatement E → ZeroBranchStatement E

lemma two_le_of_admissible {s : ℕ → ℕ} (hs : AdmissibleSubseq s) : ∀ i, 2 ≤ s i := by
  intro i
  rcases hs with ⟨hmono, h0⟩
  exact le_trans h0 (hmono.monotone (Nat.zero_le i))

lemma gap_pos (E : PrimeEnumeration) (n : ℕ) : 0 < E.gap n := by
  unfold gap
  exact Nat.sub_pos_of_lt (E.strictMono_p (Nat.lt_succ_self n))

end PrimeEnumeration