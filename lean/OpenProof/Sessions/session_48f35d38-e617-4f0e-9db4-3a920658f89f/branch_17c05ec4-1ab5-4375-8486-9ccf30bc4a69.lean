import Mathlib
/-!
OpenProof scratch session: Normalized Prime-gap Limit-point Conjecture
Session id: session_48f35d38-e617-4f0e-9db4-3a920658f89f
Mode: research
Updated: 
-/
-- branch: Normalized Prime-gap Limit-point Conjecture prover [proving]
-- branch id: branch_17c05ec4-1ab5-4375-8486-9ccf30bc4a69
-- node: Normalized Prime-gap Limit-point Conjecture [proving]
-- kind: theorem
-- statement: `PrimeEnumeration.PrimeGapLimitPointConjecture : Prop := ∀ (P : PrimeEnumeration) (C : ℝ), 0 ≤ C → ∃ s : ℕ → ℕ, StrictMono s ∧ Tendsto (fun i => P.normalizedGap (s i)) atTop (𝓝 C)`

-- artifact: Normalized Prime-gap Limit-point Conjecture prover [pending]
-- artifact id: artifact_branch_17c05ec4-1ab5-4375-8486-9ccf30bc4a69
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_17c05ec4-1ab5-4375-8486-9ccf30bc4a69
-- label: Normalized Prime-gap Limit-point Conjecture prover
-- metadata: {"hidden":false,"foreground":true}

noncomputable section

open Filter
open scoped Topology

/-- An increasing exhaustive enumeration of the prime numbers. -/
structure PrimeEnumeration where
  p : ℕ → ℕ
  prime_p : ∀ n, Nat.Prime (p n)
  strictMono_p : StrictMono p
  exhaustive_p : ∀ q : ℕ, Nat.Prime q → ∃ n, p n = q

namespace PrimeEnumeration

def gap (P : PrimeEnumeration) (n : ℕ) : ℕ :=
  P.p (n + 1) - P.p n

/-- Literal normalization from the problem statement. -/
def normalizedGap (P : PrimeEnumeration) (n : ℕ) : ℝ :=
  (P.gap n : ℝ) / Real.log (n : ℝ)

def HasLimitPoint (P : PrimeEnumeration) (C : ℝ) : Prop :=
  ∃ s : ℕ → ℕ, StrictMono s ∧
    Tendsto (fun i => P.normalizedGap (s i)) atTop (𝓝 C)

def ZeroBranch (P : PrimeEnumeration) : Prop :=
  P.HasLimitPoint 0

/-- A GPY-style encoding of the `C = 0` branch. -/
def ArbitrarilySmallNormalizedGaps (P : PrimeEnumeration) : Prop :=
  ∀ ε : ℝ, 0 < ε → ∀ N : ℕ, ∃ n ≥ N, 2 ≤ n ∧ P.normalizedGap n < ε

/-- Conjectural full limit-point statement `[0, ∞)`. -/
def PrimeGapLimitPointConjecture : Prop :=
  ∀ (P : PrimeEnumeration) (C : ℝ), 0 ≤ C → P.HasLimitPoint C

/-- Named `C = 0` specialization. -/
def ZeroBranchConjecture : Prop :=
  ∀ P : PrimeEnumeration, P.ZeroBranch

/-- Encoding of the known partial-result direction around `C = 0`. -/
def GPYZeroBranchStatement : Prop :=
  ∀ P : PrimeEnumeration, P.ArbitrarilySmallNormalizedGaps

/-- Candidate bridge for proof search. -/
def SmallGapsImplyZeroBranch : Prop :=
  ∀ P : PrimeEnumeration, P.ArbitrarilySmallNormalizedGaps → P.ZeroBranch

lemma eventually_two_le_of_strictMono {s : ℕ → ℕ} (hs : StrictMono s) :
    ∀ᶠ i in atTop, 2 ≤ s i := by
  refine Filter.eventually_atTop.2 ?_
  refine ⟨2, ?_⟩
  intro i hi
  have h01 : s 0 < s 1 := hs (by decide)
  have h12 : s 1 < s 2 := hs (by decide)
  have h2 : 2 ≤ s 2 := by
    omega
  exact le_trans h2 (hs.monotone hi)

end PrimeEnumeration