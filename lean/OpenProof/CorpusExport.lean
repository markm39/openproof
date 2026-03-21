import Lean
import Mathlib.Lean.CoreM
import Batteries.Tactic.Lint.Misc
import ImportGraph.Imports

open Lean
open Lean.Core
open System

namespace OpenProof.CorpusExport

def defaultRoots : Array Name :=
  #[`Mathlib, `Batteries, `Aesop, `Cli, `Qq, `ProofWidgets, `ImportGraph, `LeanSearchClient, `Plausible, `OpenProof]

def parseName (value : String) : Name :=
  value.splitOn "." |>.foldl (init := Name.anonymous) fun acc part => acc.str part

def parseRoots (args : List String) : Array Name := Id.run do
  let mut roots := #[]
  for arg in args do
    let trimmed := arg.trim
    if !trimmed.isEmpty then
      roots := roots.push (parseName trimmed)
  if roots.isEmpty then
    defaultRoots
  else
    roots

def shouldExportName (declName : Name) : Bool :=
  let text := declName.toString
  !declName.isInternal && (text.splitOn "._private.").length == 1

def looksLikeInstanceDecl (declName : Name) : Bool :=
  let text := declName.toString
  text.startsWith "inst" || (text.splitOn ".inst").length > 1

def levelParamsOf : ConstantInfo → Array String
  | .axiomInfo info => info.levelParams.toArray.map toString
  | .defnInfo info => info.levelParams.toArray.map toString
  | .thmInfo info => info.levelParams.toArray.map toString
  | .opaqueInfo info => info.levelParams.toArray.map toString
  | .quotInfo info => info.levelParams.toArray.map toString
  | .ctorInfo info => info.levelParams.toArray.map toString
  | .recInfo info => info.levelParams.toArray.map toString
  | .inductInfo info => info.levelParams.toArray.map toString

def isUnsafeConst : ConstantInfo → Bool
  | .axiomInfo info => info.isUnsafe
  | .defnInfo info => info.safety == .unsafe
  | .thmInfo _ => false
  | .opaqueInfo info => info.isUnsafe
  | .quotInfo _ => false
  | .ctorInfo info => info.isUnsafe
  | .recInfo info => info.isUnsafe
  | .inductInfo info => info.isUnsafe

def declKindOf (env : Environment) (declName : Name) : ConstantInfo → String
  | .thmInfo _ => "theorem"
  | .axiomInfo _ => "axiom"
  | .opaqueInfo _ => "opaque"
  | .quotInfo _ => "unknown"
  | .ctorInfo _ => "ctor"
  | .recInfo _ => "recursor"
  | .inductInfo _ =>
    if isClass env declName then
      "class"
    else if isStructure env declName then
      "structure"
    else
      "inductive"
  | .defnInfo info =>
    if looksLikeInstanceDecl declName then
      "instance"
    else if isClass env declName then
      "class"
    else if isStructure env declName then
      "structure"
    else if info.hints == ReducibilityHints.abbrev then
      "abbrev"
    else
      "def"

def isTheoremLike : ConstantInfo → Bool
  | .thmInfo _ => true
  | .axiomInfo _ => true
  | _ => false

def jsonStringArray (items : Array String) : Json :=
  Json.arr <| items.map Json.str

def printJsonLine (json : Json) : IO Unit :=
  IO.println json.compress

def moduleSourcePath? (_searchPath : Lean.SearchPath) (_moduleName : Name) : IO (Option String) :=
  pure none

def emitModuleRecord (searchPath : Lean.SearchPath) (env : Environment) (moduleName : Name) : IO Unit := do
  let imports := env.importsOf moduleName |>.map toString
  let sourcePath ← moduleSourcePath? searchPath moduleName
  printJsonLine <| Json.mkObj [
    ("recordType", "module"),
    ("moduleName", moduleName.toString),
    ("sourcePath", match sourcePath with | some path => Json.str path | none => Json.null),
    ("imports", jsonStringArray imports)
  ]

def typeText (ci : ConstantInfo) : String :=
  toString ci.type

def emitDeclarationRecord (env : Environment) (ci : ConstantInfo) : CoreM Unit := do
  let declName := ci.name
  if !shouldExportName declName then
    return
  let some moduleName := env.getModuleFor? declName | return
  let docString ← try
    pure <| (← findDocString? env declName).getD ""
  catch _ =>
    pure ""
  let declKind := declKindOf env declName ci
  if declKind == "unknown" then
    return
  let typePretty := typeText ci
  let searchText := String.intercalate " " <| [
    declName.toString,
    typePretty,
    docString,
    moduleName.toString
  ].filter fun part => !part.trim.isEmpty
  let _ ← printJsonLine <| Json.mkObj [
    ("recordType", "declaration"),
    ("declName", declName.toString),
    ("moduleName", moduleName.toString),
    ("declKind", declKind),
    ("isTheoremLike", isTheoremLike ci),
    ("isInstance", looksLikeInstanceDecl declName),
    ("isUnsafe", isUnsafeConst ci),
    ("levelParams", jsonStringArray (levelParamsOf ci)),
    ("typePretty", typePretty),
    ("docString", docString),
    ("searchText", searchText)
  ]

unsafe def exportRecords (roots : Array Name) : IO UInt32 := do
  let searchPath ← addSearchPathFromEnv (← getBuiltinSearchPath (← findSysroot))
  CoreM.withImportModules roots (searchPath := searchPath) (trustLevel := 1024) do
    let env ← getEnv
    for moduleName in env.header.moduleNames do
      let _ ← emitModuleRecord searchPath env moduleName
    for (_, ci) in env.constants do
      emitDeclarationRecord env ci
  pure 0

end OpenProof.CorpusExport

unsafe def main (args : List String) : IO UInt32 := do
  OpenProof.CorpusExport.exportRecords <| OpenProof.CorpusExport.parseRoots args
