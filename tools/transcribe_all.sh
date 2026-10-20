#!/bin/sh
# Regenerates calcpad-core/tests/xunit_*.rs from the CalcpadCE test sources.
set -e
cd "$(dirname "$0")/.."
REF=${1:-../calcpad-ref}
python3 -I tools/transcribe_xunit.py "$REF" calcpad-core/tests \
  Scalars/OperatorTests.cs Scalars/ComplexOperatorTest.cs Scalars/FunctionTests.cs Scalars/ComplexFunctionTests.cs \
  Scalars/NumericalIntegrationTests.cs Scalars/NumericalMethodsTests.cs Scalars/RootFindingTests.cs Scalars/UnitsTests.cs \
  TestCalcTests.cs MathParser/FunctionKeywordArgumentTests.cs \
  Macros/EmptyMacroArgumentTests.cs Macros/MacroKeywordArgumentTests.cs Macros/RecursiveMacroTests.cs \
  ExpressionParser/KeywordArgumentDocumentTests.cs ExpressionParser/SettingsDirectiveTests.cs \
  ExpressionParser/SubstitutionDirectiveTests.cs ExpressionParser/VisibilityDirectiveTests.cs ExpressionParser/ParseModeTests.cs \
  TrailingNewlineRenderTests.cs ExtensionMethodsTests.cs
cargo fmt
