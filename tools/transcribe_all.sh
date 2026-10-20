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
  TrailingNewlineRenderTests.cs ExtensionMethodsTests.cs \
  Matrices/ColumnMatrixOperatorInFunctionTests.cs Matrices/ColumnMatrixOperatorTests.cs \
  Matrices/DiagMatrixOperatorInFunctionTests.cs Matrices/DiagMatrixOperatorTests.cs \
  Matrices/HP/Large/HPColumnMatrixComparison.cs Matrices/HP/Large/HPColumnMatrixComparisonInFunction.cs \
  Matrices/HP/Large/HPDiagonalMatrixComparison.cs Matrices/HP/Large/HPDiagonalMatrixComparisonInFunction.cs \
  Matrices/HP/Large/HPLowerTriangularMatrixComparison.cs \
  Matrices/HP/Large/HPLowerTriangularMatrixComparisonInFunction.cs Matrices/HP/Large/HPMatrixComparison.cs \
  Matrices/HP/Large/HPMatrixComparisonInFunction.cs Matrices/HP/Large/HPSymmetricMatrixComparison.cs \
  Matrices/HP/Large/HPSymmetricMatrixComparisonInFunction.cs \
  Matrices/HP/Large/HPUpperTriangularMatrixComparison.cs \
  Matrices/HP/Large/HPUpperTriangularMatrixComparisonInFunction.cs \
  Matrices/HP/Small/HPColumnMatrixOperatorInFunctionTests.cs Matrices/HP/Small/HPColumnMatrixOperatorTests.cs \
  Matrices/HP/Small/HPDiagMatrixOperatorInFunctionTests.cs Matrices/HP/Small/HPDiagMatrixOperatorTests.cs \
  Matrices/HP/Small/HPLTriangMatrixOperatorInFunctionTests.cs Matrices/HP/Small/HPLTriangMatrixOperatorTests.cs \
  Matrices/HP/Small/HPMatrixFunctionInFunctionTests.cs Matrices/HP/Small/HPMatrixFunctionTests.cs \
  Matrices/HP/Small/HPMatrixOperatorInFunctionTests.cs Matrices/HP/Small/HPMatrixOperatorTests.cs \
  Matrices/HP/Small/HPMatrixScalarFunctionInFunctionTests.cs Matrices/HP/Small/HPMatrixScalarFunctionTests.cs \
  Matrices/HP/Small/HPSymmetricMatrixOperatorInFunctionTests.cs \
  Matrices/HP/Small/HPSymmetricMatrixOperatorTests.cs \
  Matrices/HP/Small/HPUtriangMatrixOperatorInFunctionTests.cs Matrices/HP/Small/HPUtriangMatrixOperatorTests.cs \
  Matrices/LTriangMatrixOperatorInFunctionTests.cs Matrices/LTriangMatrixOperatorTests.cs \
  Matrices/MatrixFunctionInFunctionTests.cs Matrices/MatrixFunctionTests.cs \
  Matrices/MatrixOperatorInFunctionTests.cs Matrices/MatrixOperatorTests.cs \
  Matrices/MatrixScalarFunctionInFunctionTests.cs Matrices/MatrixScalarFunctionTests.cs \
  Matrices/SymmetricMatrixOperatorInFunctionTests.cs Matrices/SymmetricMatrixOperatorTests.cs \
  Matrices/UtriangMatrixOperatorInFunctionTests.cs Matrices/UtriangMatrixOperatorTests.cs \
  Vectors/HP/HPVectorComparisonLarge.cs Vectors/HP/HPVectorFunctionInFunctionTests.cs \
  Vectors/HP/HPVectorFunctionTests.cs Vectors/HP/HPVectorOperatorInFunctionTests.cs \
  Vectors/HP/HPVectorOperatorTests.cs Vectors/HP/HPVectorScalarFunctionInFunctionTests.cs \
  Vectors/HP/HPVectorScalarFunctionTests.cs Vectors/VectorFunctionInFunctionTests.cs \
  Vectors/VectorFunctionTests.cs Vectors/VectorOperatorInFunctionTests.cs Vectors/VectorOperatorTests.cs \
  Vectors/VectorScalarFunctionInFunctionTests.cs Vectors/VectorScalarFunctionTests.cs \
  MathParser/ElementAccessTests.cs
cargo fmt
# Gate the classes listed in tools/xunit_gated.txt behind the xunit_vm feature.
grep -v '^#' tools/xunit_gated.txt | while read -r cls; do
  [ -n "$cls" ] || continue
  f="calcpad-core/tests/xunit_$cls.rs"
  [ -f "$f" ] || continue
  if ! head -1 "$f" | grep -q 'feature = "xunit_vm"'; then
    { echo '#![cfg(feature = "xunit_vm")]'; cat "$f"; } > "$f.tmp" && mv "$f.tmp" "$f"
  fi
done
