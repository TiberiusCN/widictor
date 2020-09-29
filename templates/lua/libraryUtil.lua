local export = {}

function export.checkType( name, argIdx, arg, expectType, nilOk )
end
function export.checkTypeMulti( name, argIdx, arg, expectTypes )
end
function export.checkTypeForIndex( index, value, expectType )
end
function export.checkTypeForNamedArg( name, argName, arg, expectType, nilOk )
end
function export.makeCheckSelfFunction( libraryName, varName, selfObj, selfObjDesc )
end

return export
