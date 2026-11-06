// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8,
  int32,
  float64,
  sourceText,
  sourceStartPos = 0,
  firstNonAsciiPos = 0;

const { fromCharCode } = String,
  { utf8Slice, latin1Slice } = Buffer.prototype,
  stringDecodeArrays = Array(65).fill(null);
for (let i = 0; i <= 64; i++) stringDecodeArrays[i] = Array(i).fill(0);

export function deserialize(buffer, sourceText, sourceStartPos, sourceByteLen) {
  return deserializeWith(
    buffer,
    sourceText,
    sourceStartPos,
    sourceByteLen,
    deserializeRawTransferData,
  );
}

function deserializeWith(buffer, sourceTextInput, sourceStartPosInput, sourceByteLen, deserialize) {
  uint8 = buffer;
  int32 = buffer.int32;
  float64 = buffer.float64;
  sourceText = sourceTextInput;
  sourceStartPos = sourceStartPosInput;
  // Find first non-ASCII byte in source region.
  // `sourceText.substr()` can be used for strings which are within source text and ending before
  // this position, since byte offsets equal char offsets in the all-ASCII prefix.
  // Also decode source text as Latin-1 (or reuse `sourceText` if it's all ASCII).
  firstNonAsciiPos = sourceText.length === sourceByteLen ? sourceStartPos + sourceByteLen : 0;
  let data = deserialize(int32[536870890]);
  resetBuffer();
  return data;
}

export function resetBuffer() {
  // Clear buffer and source text strings to allow them to be garbage collected
  uint8 = int32 = float64 = sourceText = void 0;
}

function deserializeProgram(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    program = {
      type: "Program",
      body: null,
      sourceType: deserializeModuleKind(pos + 137),
      hashbang: null,
      start,
      end,
    };
  program.hashbang = deserializeOptionHashbang(pos + 56);
  int32[(pos >> 2) + 22 + 2] === 0
    ? (program.body = deserializeVecStatement(pos + 112))
    : (program.body = deserializeVecDirective(pos + 88)).push(
        ...deserializeVecStatement(pos + 112),
      );
  return program;
}

function deserializeExpression(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Expression`);
  }
}

function deserializeIdentifierName(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    start,
    end,
  };
}

function deserializeIdentifierReference(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    start,
    end,
  };
}

function deserializeBindingIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    start,
    end,
  };
}

function deserializeLabelIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    start,
    end,
  };
}

function deserializeThisExpression(pos) {
  return {
    type: "ThisExpression",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeArrayExpression(pos) {
  return {
    type: "ArrayExpression",
    elements: deserializeVecArrayExpressionElement(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeArrayExpressionElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeSpreadElement(int32[(pos >> 2) + 2]);
    case 65:
      return deserializeElision(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ArrayExpressionElement`);
  }
}

function deserializeElision(pos) {
  return null;
}

function deserializeObjectExpression(pos) {
  return {
    type: "ObjectExpression",
    properties: deserializeVecObjectPropertyKind(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeObjectPropertyKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeObjectProperty(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeSpreadElement(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ObjectPropertyKind`);
  }
}

function deserializeObjectProperty(pos) {
  return {
    type: "Property",
    kind: PropertyKind_TABLE[uint8[pos + 12]],
    key: deserializePropertyKey(pos + 16),
    value: deserializeExpression(pos + 32),
    method: uint8[pos + 13] === 1,
    shorthand: uint8[pos + 14] === 1,
    computed: uint8[pos + 15] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializePropertyKey(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeIdentifierName(int32[(pos >> 2) + 2]);
    case 65:
      return deserializePrivateIdentifier(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for PropertyKey`);
  }
}

const PropertyKind_TABLE = ["init", "get", "set"];

function deserializeTemplateLiteral(pos) {
  return {
    type: "TemplateLiteral",
    quasis: deserializeVecTemplateElement(pos + 16),
    expressions: deserializeVecExpression(pos + 40),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTaggedTemplateExpression(pos) {
  return {
    type: "TaggedTemplateExpression",
    tag: deserializeExpression(pos + 16),
    quasi: deserializeTemplateLiteral(pos + 40),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTemplateElement(pos) {
  let tail = deserializeBool(pos + 12),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4);
  return {
    type: "TemplateElement",
    value: deserializeTemplateElementValue(pos + 16),
    tail,
    start,
    end,
  };
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionJSStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  return {
    type: "MemberExpression",
    object: deserializeExpression(pos + 16),
    property: deserializeExpression(pos + 32),
    optional: uint8[pos + 12] === 1,
    computed: true,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeStaticMemberExpression(pos) {
  return {
    type: "MemberExpression",
    object: deserializeExpression(pos + 16),
    property: deserializeIdentifierName(pos + 32),
    optional: uint8[pos + 12] === 1,
    computed: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializePrivateFieldExpression(pos) {
  return {
    type: "MemberExpression",
    object: deserializeExpression(pos + 16),
    property: deserializePrivateIdentifier(pos + 32),
    optional: uint8[pos + 12] === 1,
    computed: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeCallExpression(pos) {
  return {
    type: "CallExpression",
    callee: deserializeExpression(pos + 16),
    arguments: deserializeVecArgument(pos + 40),
    optional: uint8[pos + 12] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeNewExpression(pos) {
  return {
    type: "NewExpression",
    callee: deserializeExpression(pos + 16),
    arguments: deserializeVecArgument(pos + 40),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportMeta(pos) {
  let importStart = deserializeI32(pos),
    importIdent = {
      type: "Identifier",
      name: "import",
      start: importStart,
      end: deserializeI32(pos + 4) === 0 ? 0 : importStart + 6,
    },
    metaEnd = deserializeI32(pos + 4);
  return {
    type: "MetaProperty",
    meta: importIdent,
    property: {
      type: "Identifier",
      name: "meta",
      start: metaEnd === 0 ? 0 : metaEnd - 4,
      end: metaEnd,
    },
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeNewTarget(pos) {
  let newStart = deserializeI32(pos),
    newIdent = {
      type: "Identifier",
      name: "new",
      start: newStart,
      end: deserializeI32(pos + 4) === 0 ? 0 : newStart + 3,
    },
    targetEnd = deserializeI32(pos + 4);
  return {
    type: "MetaProperty",
    meta: newIdent,
    property: {
      type: "Identifier",
      name: "target",
      start: targetEnd === 0 ? 0 : targetEnd - 6,
      end: targetEnd,
    },
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeSpreadElement(pos) {
  return {
    type: "SpreadElement",
    argument: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeArgument(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeSpreadElement(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Argument`);
  }
}

function deserializeUpdateExpression(pos) {
  return {
    type: "UpdateExpression",
    operator: uint8[pos + 12] === 0 ? "++" : "--",
    prefix: uint8[pos + 13] === 1,
    argument: deserializeSimpleAssignmentTarget(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeUnaryExpression(pos) {
  return {
    type: "UnaryExpression",
    operator: UnaryOperator_TABLE[uint8[pos + 12]],
    argument: deserializeExpression(pos + 16),
    prefix: true,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBinaryExpression(pos) {
  return {
    type: "BinaryExpression",
    left: deserializeExpression(pos + 16),
    operator: BinaryOperator_TABLE[uint8[pos + 12]],
    right: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializePrivateInExpression(pos) {
  return {
    type: "BinaryExpression",
    left: deserializePrivateIdentifier(pos + 16),
    operator: "in",
    right: deserializeExpression(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeLogicalExpression(pos) {
  return {
    type: "LogicalExpression",
    left: deserializeExpression(pos + 16),
    operator: LogicalOperator_TABLE[uint8[pos + 12]],
    right: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeConditionalExpression(pos) {
  return {
    type: "ConditionalExpression",
    test: deserializeExpression(pos + 16),
    consequent: deserializeExpression(pos + 32),
    alternate: deserializeExpression(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAssignmentExpression(pos) {
  return {
    type: "AssignmentExpression",
    operator: AssignmentOperator_TABLE[uint8[pos + 12]],
    left: deserializeAssignmentTarget(pos + 16),
    right: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAssignmentTarget(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeArrayAssignmentTarget(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeObjectAssignmentTarget(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTarget`);
  }
}

function deserializeSimpleAssignmentTarget(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for SimpleAssignmentTarget`);
  }
}

function deserializeArrayAssignmentTarget(pos) {
  let elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && elements.push(rest);
  return {
    type: "ArrayPattern",
    elements,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeObjectAssignmentTarget(pos) {
  let properties = deserializeVecAssignmentTargetProperty(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && properties.push(rest);
  return {
    type: "ObjectPattern",
    properties,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAssignmentTargetRest(pos) {
  return {
    type: "RestElement",
    argument: deserializeAssignmentTarget(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAssignmentTargetMaybeDefault(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeArrayAssignmentTarget(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeObjectAssignmentTarget(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeAssignmentTargetWithDefault(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetMaybeDefault`);
  }
}

function deserializeAssignmentTargetWithDefault(pos) {
  return {
    type: "AssignmentPattern",
    left: deserializeAssignmentTarget(pos + 16),
    right: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAssignmentTargetProperty(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeAssignmentTargetPropertyIdentifier(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeAssignmentTargetPropertyProperty(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for AssignmentTargetProperty`);
  }
}

function deserializeAssignmentTargetPropertyIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    key = deserializeIdentifierReference(pos + 16),
    value = {
      type: "Identifier",
      name: key.name,
      start: key.start,
      end: key.end,
    },
    init = deserializeOptionExpression(pos + 48);
  init !== null
    && (value = {
      type: "AssignmentPattern",
      left: value,
      right: init,
      start,
      end,
    });
  return {
    type: "Property",
    kind: "init",
    key,
    value,
    method: false,
    shorthand: true,
    computed: false,
    start,
    end,
  };
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  return {
    type: "Property",
    kind: "init",
    key: deserializePropertyKey(pos + 16),
    value: deserializeAssignmentTargetMaybeDefault(pos + 32),
    method: false,
    shorthand: false,
    computed: uint8[pos + 12] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeSequenceExpression(pos) {
  return {
    type: "SequenceExpression",
    expressions: deserializeVecExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeSuper(pos) {
  return {
    type: "Super",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAwaitExpression(pos) {
  return {
    type: "AwaitExpression",
    argument: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeChainExpression(pos) {
  return {
    type: "ChainExpression",
    expression: deserializeChainElement(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeChainElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ChainElement`);
  }
}

function deserializeParenthesizedExpression(pos) {
  let node;
  node = {
    type: "ParenthesizedExpression",
    expression: null,
    start: deserializeI32(pos),
    end: deserializeI32(pos + 4),
  };
  node.expression = deserializeExpression(pos + 16);
  return node;
}

function deserializeStatement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBlockStatement(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeBreakStatement(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeContinueStatement(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeDebuggerStatement(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeDoWhileStatement(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeEmptyStatement(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeExpressionStatement(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeForInStatement(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeForOfStatement(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeForStatement(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeIfStatement(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeLabeledStatement(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeReturnStatement(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeSwitchStatement(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeThrowStatement(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeTryStatement(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeWhileStatement(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeWithStatement(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeVariableDeclaration(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSTypeAliasDeclaration(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSInterfaceDeclaration(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSEnumDeclaration(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSExternalModuleDeclaration(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSNamespaceDeclaration(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeTSGlobalDeclaration(int32[(pos >> 2) + 2]);
    case 41:
      return deserializeTSImportEqualsDeclaration(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeImportDeclaration(int32[(pos >> 2) + 2]);
    case 65:
      return deserializeExportAllDeclaration(int32[(pos >> 2) + 2]);
    case 66:
      return deserializeExportDefaultDeclaration(int32[(pos >> 2) + 2]);
    case 67:
      return deserializeExportDeclaration(int32[(pos >> 2) + 2]);
    case 68:
      return deserializeExportNamedDeclaration(int32[(pos >> 2) + 2]);
    case 69:
      return deserializeExportFromDeclaration(int32[(pos >> 2) + 2]);
    case 70:
      return deserializeTSExportAssignment(int32[(pos >> 2) + 2]);
    case 71:
      return deserializeTSNamespaceExportDeclaration(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Statement`);
  }
}

function deserializeDirective(pos) {
  return {
    type: "ExpressionStatement",
    expression: deserializeStringLiteral(pos + 16),
    directive: deserializeStr(pos + 64),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeHashbang(pos) {
  return {
    type: "Hashbang",
    value: deserializeStr(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBlockStatement(pos) {
  return {
    type: "BlockStatement",
    body: deserializeVecStatement(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeDeclaration(pos) {
  switch (uint8[pos]) {
    case 32:
      return deserializeVariableDeclaration(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSTypeAliasDeclaration(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSInterfaceDeclaration(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSEnumDeclaration(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSExternalModuleDeclaration(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSNamespaceDeclaration(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeTSGlobalDeclaration(int32[(pos >> 2) + 2]);
    case 41:
      return deserializeTSImportEqualsDeclaration(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for Declaration`);
  }
}

function deserializeVariableDeclaration(pos) {
  return {
    type: "VariableDeclaration",
    kind: VariableDeclarationKind_TABLE[uint8[pos + 12]],
    declarations: deserializeVecVariableDeclarator(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

const VariableDeclarationKind_TABLE = ["var", "let", "const", "using", "await using"];

function deserializeVariableDeclarator(pos) {
  return {
    type: "VariableDeclarator",
    id: deserializeBindingPattern(pos + 16),
    init: deserializeOptionExpression(pos + 40),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeEmptyStatement(pos) {
  return {
    type: "EmptyStatement",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExpressionStatement(pos) {
  return {
    type: "ExpressionStatement",
    expression: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeIfStatement(pos) {
  return {
    type: "IfStatement",
    test: deserializeExpression(pos + 16),
    consequent: deserializeStatement(pos + 32),
    alternate: deserializeOptionStatement(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeDoWhileStatement(pos) {
  return {
    type: "DoWhileStatement",
    body: deserializeStatement(pos + 16),
    test: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeWhileStatement(pos) {
  return {
    type: "WhileStatement",
    test: deserializeExpression(pos + 16),
    body: deserializeStatement(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeForStatement(pos) {
  return {
    type: "ForStatement",
    init: deserializeOptionForStatementInit(pos + 16),
    test: deserializeOptionExpression(pos + 32),
    update: deserializeOptionExpression(pos + 48),
    body: deserializeStatement(pos + 64),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeForStatementInit(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeVariableDeclaration(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ForStatementInit`);
  }
}

function deserializeForInStatement(pos) {
  return {
    type: "ForInStatement",
    left: deserializeForStatementLeft(pos + 16),
    right: deserializeExpression(pos + 32),
    body: deserializeStatement(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeForStatementLeft(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeArrayAssignmentTarget(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeObjectAssignmentTarget(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeVariableDeclaration(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ForStatementLeft`);
  }
}

function deserializeForOfStatement(pos) {
  return {
    type: "ForOfStatement",
    await: uint8[pos + 64] === 1,
    left: deserializeForStatementLeft(pos + 16),
    right: deserializeExpression(pos + 32),
    body: deserializeStatement(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeContinueStatement(pos) {
  return {
    type: "ContinueStatement",
    label: deserializeOptionLabelIdentifier(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBreakStatement(pos) {
  return {
    type: "BreakStatement",
    label: deserializeOptionLabelIdentifier(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeReturnStatement(pos) {
  return {
    type: "ReturnStatement",
    argument: deserializeOptionExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeWithStatement(pos) {
  return {
    type: "WithStatement",
    object: deserializeExpression(pos + 16),
    body: deserializeStatement(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeSwitchStatement(pos) {
  return {
    type: "SwitchStatement",
    discriminant: deserializeExpression(pos + 16),
    cases: deserializeVecSwitchCase(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeSwitchCase(pos) {
  return {
    type: "SwitchCase",
    test: deserializeOptionExpression(pos + 16),
    consequent: deserializeVecStatement(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeLabeledStatement(pos) {
  return {
    type: "LabeledStatement",
    label: deserializeLabelIdentifier(pos + 16),
    body: deserializeStatement(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeThrowStatement(pos) {
  return {
    type: "ThrowStatement",
    argument: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTryStatement(pos) {
  return {
    type: "TryStatement",
    block: deserializeBlockStatement(int32[(pos >> 2) + 4]),
    handler: deserializeOptionBoxCatchClause(pos + 24),
    finalizer: deserializeOptionBoxBlockStatement(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeCatchClause(pos) {
  return {
    type: "CatchClause",
    param: deserializeOptionCatchParameter(pos + 16),
    body: deserializeBlockStatement(int32[(pos >> 2) + 14]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeCatchParameter(pos) {
  return deserializeBindingPattern(pos + 16);
}

function deserializeDebuggerStatement(pos) {
  return {
    type: "DebuggerStatement",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBindingPattern(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBindingIdentifier(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeObjectPattern(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeArrayPattern(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeAssignmentPattern(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for BindingPattern`);
  }
}

function deserializeAssignmentPattern(pos) {
  return {
    type: "AssignmentPattern",
    left: deserializeBindingPattern(pos + 16),
    right: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeObjectPattern(pos) {
  let properties = deserializeVecBindingProperty(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && properties.push(rest);
  return {
    type: "ObjectPattern",
    properties,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBindingProperty(pos) {
  return {
    type: "Property",
    kind: "init",
    key: deserializePropertyKey(pos + 16),
    value: deserializeBindingPattern(pos + 32),
    method: false,
    shorthand: uint8[pos + 12] === 1,
    computed: uint8[pos + 13] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeArrayPattern(pos) {
  let elements = deserializeVecOptionBindingPattern(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && elements.push(rest);
  return {
    type: "ArrayPattern",
    elements,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBindingRestElement(pos) {
  return {
    type: "RestElement",
    argument: deserializeBindingPattern(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeFunction(pos) {
  let params = deserializeBoxFormalParameters(pos + 64);
  return {
    type: FunctionType_TABLE[uint8[pos + 88]],
    id: deserializeOptionBindingIdentifier(pos + 16),
    generator: uint8[pos + 89] === 1,
    async: uint8[pos + 90] === 1,
    params,
    body: deserializeOptionBoxFunctionBody(pos + 80),
    expression: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

const FunctionType_TABLE = [
  "FunctionDeclaration",
  "FunctionExpression",
  "TSDeclareFunction",
  "TSEmptyBodyFunctionExpression",
];

function deserializeFormalParameters(pos) {
  let params = deserializeVecFormalParameter(pos + 16),
    restFieldPos32 = (pos >> 2) + 10;
  if (int32[restFieldPos32] !== 0 && int32[restFieldPos32 + 1] !== 0) {
    pos = int32[restFieldPos32];
    let rest = {
      type: "RestElement",
      argument: null,
      start: deserializeI32(pos + 40),
      end: deserializeI32(pos + 44),
    };
    rest.argument = deserializeBindingPattern(pos + 56);
    params.push(rest);
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param,
    initializerFieldPos32 = (pos >> 2) + 16;
  if (int32[initializerFieldPos32] !== 0 && int32[initializerFieldPos32 + 1] !== 0) {
    param = {
      type: "AssignmentPattern",
      left: null,
      right: null,
      start: deserializeI32(pos),
      end: deserializeI32(pos + 4),
    };
    param.left = deserializeBindingPattern(pos + 40);
    param.right = deserializeOptionBoxExpression(pos + 64);
  } else param = deserializeBindingPattern(pos + 40);
  return param;
}

function deserializeFunctionBody(pos) {
  let body;
  if (int32[(pos >> 2) + 6] === 0) body = deserializeVecStatement(pos + 40);
  else {
    body = deserializeVecDirective(pos + 16);
    body.push(...deserializeVecStatement(pos + 40));
  }
  return {
    type: "BlockStatement",
    body,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeArrowFunctionBody(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeFunctionBody(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ArrowFunctionBody`);
  }
}

function deserializeArrowFunctionExpression(pos) {
  return {
    type: "ArrowFunctionExpression",
    expression: uint8[pos + 40] !== 64,
    async: uint8[pos + 56] === 1,
    params: deserializeFormalParameters(int32[(pos >> 2) + 6]),
    body: deserializeArrowFunctionBody(pos + 40),
    id: null,
    generator: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeYieldExpression(pos) {
  return {
    type: "YieldExpression",
    delegate: uint8[pos + 12] === 1,
    argument: deserializeOptionExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeClass(pos) {
  let superClass = deserializeOptionExpression(pos + 80);
  return {
    type: uint8[pos + 136] === 0 ? "ClassDeclaration" : "ClassExpression",
    decorators: deserializeVecDecorator(pos + 16),
    id: deserializeOptionBindingIdentifier(pos + 40),
    superClass,
    body: deserializeClassBody(int32[(pos >> 2) + 32]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeClassBody(pos) {
  return {
    type: "ClassBody",
    body: deserializeVecClassElement(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeClassElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeStaticBlock(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeMethodDefinition(int32[(pos >> 2) + 2]);
    case 2:
      return deserializePropertyDefinition(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeAccessorProperty(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSIndexSignature(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ClassElement`);
  }
}

function deserializeMethodDefinition(pos) {
  return {
    type: uint8[pos + 12] === 0 ? "MethodDefinition" : "TSAbstractMethodDefinition",
    decorators: deserializeVecDecorator(pos + 16),
    key: deserializePropertyKey(pos + 40),
    value: deserializeFunction(int32[(pos >> 2) + 14]),
    kind: MethodDefinitionKind_TABLE[uint8[pos + 13]],
    computed: uint8[pos + 14] === 1,
    static: uint8[pos + 15] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializePropertyDefinition(pos) {
  return {
    type: uint8[pos + 12] === 0 ? "PropertyDefinition" : "TSAbstractPropertyDefinition",
    decorators: deserializeVecDecorator(pos + 16),
    key: deserializePropertyKey(pos + 40),
    value: deserializeOptionExpression(pos + 64),
    computed: uint8[pos + 13] === 1,
    static: uint8[pos + 14] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

const MethodDefinitionKind_TABLE = ["constructor", "method", "get", "set"];

function deserializePrivateIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "PrivateIdentifier",
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start + 1, end)
        : deserializeStr(pos + 16),
    start,
    end,
  };
}

function deserializeStaticBlock(pos) {
  return {
    type: "StaticBlock",
    body: deserializeVecStatement(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeAccessorProperty(pos) {
  return {
    type: uint8[pos + 12] === 0 ? "AccessorProperty" : "TSAbstractAccessorProperty",
    decorators: deserializeVecDecorator(pos + 16),
    key: deserializePropertyKey(pos + 40),
    value: deserializeOptionExpression(pos + 64),
    computed: uint8[pos + 13] === 1,
    static: uint8[pos + 14] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportExpression(pos) {
  return {
    type: "ImportExpression",
    source: deserializeExpression(pos + 16),
    options: deserializeOptionExpression(pos + 32),
    phase: deserializeOptionImportPhase(pos + 12),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportDeclaration(pos) {
  let specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 16);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 88);
  return {
    type: "ImportDeclaration",
    specifiers,
    source: deserializeStringLiteral(pos + 40),
    phase: deserializeOptionImportPhase(pos + 12),
    attributes: withClause === null ? [] : withClause.attributes,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportPhase(pos) {
  switch (uint8[pos]) {
    case 0:
      return "source";
    case 1:
      return "defer";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportPhase`);
  }
}

function deserializeImportDeclarationSpecifier(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeImportSpecifier(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeImportDefaultSpecifier(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeImportNamespaceSpecifier(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportDeclarationSpecifier`);
  }
}

function deserializeImportSpecifier(pos) {
  return {
    type: "ImportSpecifier",
    imported: deserializeModuleExportName(pos + 16),
    local: deserializeBindingIdentifier(pos + 72),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportDefaultSpecifier(pos) {
  return {
    type: "ImportDefaultSpecifier",
    local: deserializeBindingIdentifier(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportNamespaceSpecifier(pos) {
  return {
    type: "ImportNamespaceSpecifier",
    local: deserializeBindingIdentifier(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 16) };
}

function deserializeImportAttribute(pos) {
  return {
    type: "ImportAttribute",
    key: deserializeImportAttributeKey(pos + 16),
    value: deserializeStringLiteral(pos + 72),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportAttributeKey(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(pos + 8);
    case 1:
      return deserializeStringLiteral(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportAttributeKey`);
  }
}

function deserializeExportDeclaration(pos) {
  return {
    type: "ExportNamedDeclaration",
    declaration: deserializeDeclaration(pos + 16),
    specifiers: [],
    source: null,
    attributes: [],
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportNamedDeclaration(pos) {
  return {
    type: "ExportNamedDeclaration",
    declaration: null,
    specifiers: deserializeVecExportSpecifier(pos + 16),
    source: null,
    attributes: [],
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportFromDeclaration(pos) {
  let withClause = deserializeOptionBoxWithClause(pos + 88);
  return {
    type: "ExportNamedDeclaration",
    declaration: null,
    specifiers: deserializeVecExportSpecifier(pos + 16),
    source: deserializeStringLiteral(pos + 40),
    attributes: withClause === null ? [] : withClause.attributes,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportDefaultDeclaration(pos) {
  return {
    type: "ExportDefaultDeclaration",
    declaration: deserializeExportDefaultDeclarationKind(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportAllDeclaration(pos) {
  let withClause = deserializeOptionBoxWithClause(pos + 120);
  return {
    type: "ExportAllDeclaration",
    exported: deserializeOptionModuleExportName(pos + 16),
    source: deserializeStringLiteral(pos + 72),
    attributes: withClause === null ? [] : withClause.attributes,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportSpecifier(pos) {
  return {
    type: "ExportSpecifier",
    local: deserializeModuleExportName(pos + 16),
    exported: deserializeModuleExportName(pos + 72),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportDefaultDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 65:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 66:
      return deserializeTSInterfaceDeclaration(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportDefaultDeclarationKind`);
  }
}

function deserializeModuleExportName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(pos + 8);
    case 1:
      return deserializeIdentifierReference(pos + 8);
    case 2:
      return deserializeStringLiteral(pos + 8);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ModuleExportName`);
  }
}

function deserializeV8IntrinsicExpression(pos) {
  return {
    type: "V8IntrinsicExpression",
    name: deserializeIdentifierName(pos + 16),
    arguments: deserializeVecArgument(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeBooleanLiteral(pos) {
  let value = uint8[pos + 12] === 1,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Literal",
    value,
    raw: start === 0 && end === 0 ? null : value ? "true" : "false",
    start,
    end,
  };
}

function deserializeNullLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Literal",
    value: null,
    raw: start === 0 && end === 0 ? null : "null",
    start,
    end,
  };
}

function deserializeNumericLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Literal",
    value: float64[(pos >> 3) + 4],
    raw:
      int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
        ? null
        : sourceText.slice(start, end),
    start,
    end,
  };
}

function deserializeStringLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Literal",
    value:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start + 1, end - 1)
        : deserializeJSStr(pos + 16),
    raw:
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end),
    start,
    end,
  };
}

function deserializeBigIntLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    bigint = deserializeStr(pos + 16);
  return {
    type: "Literal",
    value: BigInt(bigint),
    raw:
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end),
    bigint,
    start,
    end,
  };
}

function deserializeRegExpLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    regex = deserializeRegExp(pos + 16),
    value = null;
  try {
    value = new RegExp(regex.pattern, regex.flags);
  } catch {}
  return {
    type: "Literal",
    value,
    raw:
      int32[(pos >> 2) + 12] === 0 && int32[(pos >> 2) + 13] === 0
        ? null
        : sourceText.slice(start, end),
    regex,
    start,
    end,
  };
}

function deserializeRegExp(pos) {
  return {
    pattern: deserializeStr(pos),
    flags: deserializeRegExpFlags(pos + 24),
  };
}

function deserializeRegExpFlags(pos) {
  let flagBits = deserializeU8(pos),
    flags = "";
  // Alphabetical order
  flagBits & 64 && (flags += "d");
  flagBits & 1 && (flags += "g");
  flagBits & 2 && (flags += "i");
  flagBits & 4 && (flags += "m");
  flagBits & 8 && (flags += "s");
  flagBits & 16 && (flags += "u");
  flagBits & 128 && (flags += "v");
  flagBits & 32 && (flags += "y");
  return flags;
}

function deserializeJSXElement(pos) {
  let closingElement = deserializeOptionBoxJSXClosingElement(pos + 48),
    openingElement = deserializeBoxJSXOpeningElement(pos + 16);
  closingElement === null && (openingElement.selfClosing = true);
  return {
    type: "JSXElement",
    openingElement,
    children: deserializeVecJSXChild(pos + 24),
    closingElement,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXOpeningElement(pos) {
  return {
    type: "JSXOpeningElement",
    name: deserializeJSXElementName(pos + 16),
    attributes: deserializeVecJSXAttributeItem(pos + 40),
    selfClosing: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXClosingElement(pos) {
  return {
    type: "JSXClosingElement",
    name: deserializeJSXElementName(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXFragment(pos) {
  return {
    type: "JSXFragment",
    openingFragment: deserializeJSXOpeningFragment(pos + 16),
    children: deserializeVecJSXChild(pos + 32),
    closingFragment: deserializeJSXClosingFragment(pos + 56),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXOpeningFragment(pos) {
  return {
    type: "JSXOpeningFragment",
    attributes: [],
    selfClosing: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXClosingFragment(pos) {
  return {
    type: "JSXClosingFragment",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXElementName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXIdentifier(int32[(pos >> 2) + 2]);
    case 1:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return {
        type: "JSXIdentifier",
        name: ident.name,
        start: ident.start,
        end: ident.end,
      };
    case 2:
      return deserializeJSXNamespacedName(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeJSXMemberExpression(int32[(pos >> 2) + 2]);
    case 4:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return {
        type: "JSXIdentifier",
        name: "this",
        start: thisExpr.start,
        end: thisExpr.end,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  return {
    type: "JSXNamespacedName",
    namespace: deserializeJSXIdentifier(pos + 16),
    name: deserializeJSXIdentifier(pos + 48),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXMemberExpression(pos) {
  return {
    type: "JSXMemberExpression",
    object: deserializeJSXMemberExpressionObject(pos + 16),
    property: deserializeJSXIdentifier(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXMemberExpressionObject(pos) {
  switch (uint8[pos]) {
    case 0:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return {
        type: "JSXIdentifier",
        name: ident.name,
        start: ident.start,
        end: ident.end,
      };
    case 1:
      return deserializeJSXMemberExpression(int32[(pos >> 2) + 2]);
    case 2:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return {
        type: "JSXIdentifier",
        name: "this",
        start: thisExpr.start,
        end: thisExpr.end,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  return {
    type: "JSXExpressionContainer",
    expression: deserializeJSXExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXExpression(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNullLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeRegExpLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeSuper(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeArrayExpression(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeArrowFunctionExpression(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeAssignmentExpression(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeAwaitExpression(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeBinaryExpression(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeCallExpression(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeChainExpression(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeClass(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeConditionalExpression(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeFunction(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeImportExpression(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeLogicalExpression(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeNewExpression(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeObjectExpression(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeParenthesizedExpression(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeSequenceExpression(int32[(pos >> 2) + 2]);
    case 25:
      return deserializeTaggedTemplateExpression(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeUpdateExpression(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeYieldExpression(int32[(pos >> 2) + 2]);
    case 30:
      return deserializePrivateInExpression(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeImportMeta(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeNewTarget(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeTSAsExpression(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeTSSatisfiesExpression(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeTSTypeAssertion(int32[(pos >> 2) + 2]);
    case 38:
      return deserializeTSNonNullExpression(int32[(pos >> 2) + 2]);
    case 39:
      return deserializeTSInstantiationExpression(int32[(pos >> 2) + 2]);
    case 40:
      return deserializeV8IntrinsicExpression(int32[(pos >> 2) + 2]);
    case 48:
      return deserializeComputedMemberExpression(int32[(pos >> 2) + 2]);
    case 49:
      return deserializeStaticMemberExpression(int32[(pos >> 2) + 2]);
    case 50:
      return deserializePrivateFieldExpression(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeJSXEmptyExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXExpression`);
  }
}

function deserializeJSXEmptyExpression(pos) {
  return {
    type: "JSXEmptyExpression",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXAttributeItem(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXAttribute(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeJSXSpreadAttribute(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeItem`);
  }
}

function deserializeJSXAttribute(pos) {
  return {
    type: "JSXAttribute",
    name: deserializeJSXAttributeName(pos + 16),
    value: deserializeOptionJSXAttributeValue(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXSpreadAttribute(pos) {
  return {
    type: "JSXSpreadAttribute",
    argument: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXAttributeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXIdentifier(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeJSXNamespacedName(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeName`);
  }
}

function deserializeJSXAttributeValue(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeJSXExpressionContainer(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXAttributeValue`);
  }
}

function deserializeJSXIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "JSXIdentifier",
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    start,
    end,
  };
}

function deserializeJSXChild(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXText(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeJSXElement(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeJSXFragment(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeJSXExpressionContainer(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeJSXSpreadChild(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXChild`);
  }
}

function deserializeJSXSpreadChild(pos) {
  return {
    type: "JSXSpreadChild",
    expression: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSXText(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "JSXText",
    value:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    raw:
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end),
    start,
    end,
  };
}

function deserializeTSThisParameter(pos) {
  return {
    type: "Identifier",
    decorators: [],
    name: "this",
    optional: false,
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 24),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSEnumDeclaration(pos) {
  return {
    type: "TSEnumDeclaration",
    id: deserializeBindingIdentifier(pos + 16),
    body: deserializeTSEnumBody(pos + 48),
    const: uint8[pos + 12] === 1,
    declare: uint8[pos + 13] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSEnumBody(pos) {
  return {
    type: "TSEnumBody",
    members: deserializeVecTSEnumMember(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSEnumMember(pos) {
  return {
    type: "TSEnumMember",
    id: deserializeTSEnumMemberName(pos + 16),
    initializer: deserializeOptionExpression(pos + 32),
    computed: deserializeU8(pos + 16) > 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSEnumMemberName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSEnumMemberName`);
  }
}

function deserializeTSTypeAnnotation(pos) {
  return {
    type: "TSTypeAnnotation",
    typeAnnotation: deserializeTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSLiteralType(pos) {
  return {
    type: "TSLiteralType",
    literal: deserializeTSLiteral(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSLiteral(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeBooleanLiteral(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeNumericLiteral(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeBigIntLiteral(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeStringLiteral(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTemplateLiteral(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeUnaryExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSLiteral`);
  }
}

function deserializeTSType(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSAnyKeyword(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSBigIntKeyword(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSBooleanKeyword(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSIntrinsicKeyword(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSNeverKeyword(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeTSNullKeyword(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTSNumberKeyword(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeTSObjectKeyword(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeTSStringKeyword(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeTSSymbolKeyword(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeTSThisType(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeTSUndefinedKeyword(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeTSUnknownKeyword(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeTSVoidKeyword(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeTSArrayType(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeTSConditionalType(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeTSConstructorType(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeTSFunctionType(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeTSImportType(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeTSIndexedAccessType(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeTSInferType(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeTSIntersectionType(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeTSLiteralType(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeTSMappedType(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeTSNamedTupleMember(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeTSTemplateLiteralType(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeTSTupleType(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeTSTypeLiteral(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeTSTypeOperator(int32[(pos >> 2) + 2]);
    case 30:
      return deserializeTSTypePredicate(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeTSTypeQuery(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeTSTypeReference(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeTSUnionType(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeTSParenthesizedType(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeJSDocNullableType(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeJSDocNonNullableType(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeJSDocUnknownType(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSType`);
  }
}

function deserializeTSConditionalType(pos) {
  return {
    type: "TSConditionalType",
    checkType: deserializeTSType(pos + 16),
    extendsType: deserializeTSType(pos + 32),
    trueType: deserializeTSType(pos + 48),
    falseType: deserializeTSType(pos + 64),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSUnionType(pos) {
  return {
    type: "TSUnionType",
    types: deserializeVecTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSIntersectionType(pos) {
  return {
    type: "TSIntersectionType",
    types: deserializeVecTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSParenthesizedType(pos) {
  let node;
  node = {
    type: "TSParenthesizedType",
    typeAnnotation: null,
    start: deserializeI32(pos),
    end: deserializeI32(pos + 4),
  };
  node.typeAnnotation = deserializeTSType(pos + 16);
  return node;
}

function deserializeTSTypeOperator(pos) {
  return {
    type: "TSTypeOperator",
    operator: TSTypeOperatorOperator_TABLE[uint8[pos + 12]],
    typeAnnotation: deserializeTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

const TSTypeOperatorOperator_TABLE = ["keyof", "unique", "readonly"];

function deserializeTSArrayType(pos) {
  return {
    type: "TSArrayType",
    elementType: deserializeTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSIndexedAccessType(pos) {
  return {
    type: "TSIndexedAccessType",
    objectType: deserializeTSType(pos + 16),
    indexType: deserializeTSType(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTupleType(pos) {
  return {
    type: "TSTupleType",
    elementTypes: deserializeVecTSTupleElement(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSNamedTupleMember(pos) {
  return {
    type: "TSNamedTupleMember",
    label: deserializeIdentifierName(pos + 16),
    elementType: deserializeTSTupleElement(pos + 48),
    optional: uint8[pos + 12] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSOptionalType(pos) {
  return {
    type: "TSOptionalType",
    typeAnnotation: deserializeTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSRestType(pos) {
  return {
    type: "TSRestType",
    typeAnnotation: deserializeTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTupleElement(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSAnyKeyword(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSBigIntKeyword(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSBooleanKeyword(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSIntrinsicKeyword(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSNeverKeyword(int32[(pos >> 2) + 2]);
    case 5:
      return deserializeTSNullKeyword(int32[(pos >> 2) + 2]);
    case 6:
      return deserializeTSNumberKeyword(int32[(pos >> 2) + 2]);
    case 7:
      return deserializeTSObjectKeyword(int32[(pos >> 2) + 2]);
    case 8:
      return deserializeTSStringKeyword(int32[(pos >> 2) + 2]);
    case 9:
      return deserializeTSSymbolKeyword(int32[(pos >> 2) + 2]);
    case 10:
      return deserializeTSThisType(int32[(pos >> 2) + 2]);
    case 11:
      return deserializeTSUndefinedKeyword(int32[(pos >> 2) + 2]);
    case 12:
      return deserializeTSUnknownKeyword(int32[(pos >> 2) + 2]);
    case 13:
      return deserializeTSVoidKeyword(int32[(pos >> 2) + 2]);
    case 14:
      return deserializeTSArrayType(int32[(pos >> 2) + 2]);
    case 15:
      return deserializeTSConditionalType(int32[(pos >> 2) + 2]);
    case 16:
      return deserializeTSConstructorType(int32[(pos >> 2) + 2]);
    case 17:
      return deserializeTSFunctionType(int32[(pos >> 2) + 2]);
    case 18:
      return deserializeTSImportType(int32[(pos >> 2) + 2]);
    case 19:
      return deserializeTSIndexedAccessType(int32[(pos >> 2) + 2]);
    case 20:
      return deserializeTSInferType(int32[(pos >> 2) + 2]);
    case 21:
      return deserializeTSIntersectionType(int32[(pos >> 2) + 2]);
    case 22:
      return deserializeTSLiteralType(int32[(pos >> 2) + 2]);
    case 23:
      return deserializeTSMappedType(int32[(pos >> 2) + 2]);
    case 24:
      return deserializeTSNamedTupleMember(int32[(pos >> 2) + 2]);
    case 26:
      return deserializeTSTemplateLiteralType(int32[(pos >> 2) + 2]);
    case 27:
      return deserializeTSTupleType(int32[(pos >> 2) + 2]);
    case 28:
      return deserializeTSTypeLiteral(int32[(pos >> 2) + 2]);
    case 29:
      return deserializeTSTypeOperator(int32[(pos >> 2) + 2]);
    case 30:
      return deserializeTSTypePredicate(int32[(pos >> 2) + 2]);
    case 31:
      return deserializeTSTypeQuery(int32[(pos >> 2) + 2]);
    case 32:
      return deserializeTSTypeReference(int32[(pos >> 2) + 2]);
    case 33:
      return deserializeTSUnionType(int32[(pos >> 2) + 2]);
    case 34:
      return deserializeTSParenthesizedType(int32[(pos >> 2) + 2]);
    case 35:
      return deserializeJSDocNullableType(int32[(pos >> 2) + 2]);
    case 36:
      return deserializeJSDocNonNullableType(int32[(pos >> 2) + 2]);
    case 37:
      return deserializeJSDocUnknownType(int32[(pos >> 2) + 2]);
    case 64:
      return deserializeTSOptionalType(int32[(pos >> 2) + 2]);
    case 65:
      return deserializeTSRestType(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTupleElement`);
  }
}

function deserializeTSAnyKeyword(pos) {
  return {
    type: "TSAnyKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSStringKeyword(pos) {
  return {
    type: "TSStringKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSBooleanKeyword(pos) {
  return {
    type: "TSBooleanKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSNumberKeyword(pos) {
  return {
    type: "TSNumberKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSNeverKeyword(pos) {
  return {
    type: "TSNeverKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  return {
    type: "TSIntrinsicKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSUnknownKeyword(pos) {
  return {
    type: "TSUnknownKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSNullKeyword(pos) {
  return {
    type: "TSNullKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSUndefinedKeyword(pos) {
  return {
    type: "TSUndefinedKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSVoidKeyword(pos) {
  return {
    type: "TSVoidKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSSymbolKeyword(pos) {
  return {
    type: "TSSymbolKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSThisType(pos) {
  return {
    type: "TSThisType",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSObjectKeyword(pos) {
  return {
    type: "TSObjectKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSBigIntKeyword(pos) {
  return {
    type: "TSBigIntKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeReference(pos) {
  return {
    type: "TSTypeReference",
    typeName: deserializeTSTypeName(pos + 16),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSQualifiedName(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeName`);
  }
}

function deserializeTSQualifiedName(pos) {
  return {
    type: "TSQualifiedName",
    left: deserializeTSTypeName(pos + 16),
    right: deserializeIdentifierName(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeParameterInstantiation(pos) {
  return {
    type: "TSTypeParameterInstantiation",
    params: deserializeVecTSType(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeParameter(pos) {
  return {
    type: "TSTypeParameter",
    name: deserializeBindingIdentifier(pos + 16),
    constraint: deserializeOptionTSType(pos + 48),
    default: deserializeOptionTSType(pos + 64),
    in: uint8[pos + 12] === 1,
    out: uint8[pos + 13] === 1,
    const: uint8[pos + 14] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeParameterDeclaration(pos) {
  return {
    type: "TSTypeParameterDeclaration",
    params: deserializeVecTSTypeParameter(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeAliasDeclaration(pos) {
  return {
    type: "TSTypeAliasDeclaration",
    id: deserializeBindingIdentifier(pos + 16),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 48),
    typeAnnotation: deserializeTSType(pos + 56),
    declare: uint8[pos + 72] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSInterfaceDeclaration(pos) {
  return {
    type: "TSInterfaceDeclaration",
    id: deserializeBindingIdentifier(pos + 16),
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 48),
    extends: deserializeVecTSInterfaceHeritage(pos + 56),
    body: deserializeTSInterfaceBody(int32[(pos >> 2) + 20]),
    declare: uint8[pos + 88] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSInterfaceBody(pos) {
  return {
    type: "TSInterfaceBody",
    body: deserializeVecTSSignature(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSPropertySignature(pos) {
  return {
    type: "TSPropertySignature",
    computed: uint8[pos + 12] === 1,
    optional: uint8[pos + 13] === 1,
    readonly: uint8[pos + 14] === 1,
    key: deserializePropertyKey(pos + 16),
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 32),
    accessibility: null,
    static: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSSignature(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSIndexSignature(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSPropertySignature(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSCallSignatureDeclaration(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSConstructSignatureDeclaration(int32[(pos >> 2) + 2]);
    case 4:
      return deserializeTSMethodSignature(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSSignature`);
  }
}

function deserializeTSIndexSignature(pos) {
  return {
    type: "TSIndexSignature",
    parameters: [deserializeTSIndexSignatureName(pos + 16)],
    typeAnnotation: deserializeTSTypeAnnotation(int32[(pos >> 2) + 14]),
    readonly: uint8[pos + 12] === 1,
    static: uint8[pos + 13] === 1,
    accessibility: null,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSCallSignatureDeclaration(pos) {
  let params = deserializeBoxFormalParameters(pos + 32),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  return {
    type: "TSCallSignatureDeclaration",
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 16),
    params,
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 40),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

const TSMethodSignatureKind_TABLE = ["method", "get", "set"];

function deserializeTSMethodSignature(pos) {
  let params = deserializeBoxFormalParameters(pos + 48),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 40);
  thisParam !== null && params.unshift(thisParam);
  return {
    type: "TSMethodSignature",
    key: deserializePropertyKey(pos + 16),
    computed: uint8[pos + 64] === 1,
    optional: uint8[pos + 65] === 1,
    kind: TSMethodSignatureKind_TABLE[uint8[pos + 66]],
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 32),
    params,
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 56),
    accessibility: null,
    readonly: false,
    static: false,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSConstructSignatureDeclaration(pos) {
  return {
    type: "TSConstructSignatureDeclaration",
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 16),
    params: deserializeFormalParameters(int32[(pos >> 2) + 6]),
    returnType: deserializeOptionBoxTSTypeAnnotation(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSIndexSignatureName(pos) {
  return {
    type: "Identifier",
    decorators: [],
    name: deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: deserializeTSTypeAnnotation(int32[(pos >> 2) + 8]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSInterfaceHeritage(pos) {
  let expression = deserializeTSTypeName(pos + 16);
  if (expression.type === "TSQualifiedName") {
    let object = expression.left,
      { right } = expression,
      previous = (expression = {
        type: "MemberExpression",
        object,
        property: right,
        optional: false,
        computed: false,
        start: expression.start,
        end: expression.end,
      });
    for (; object.type === "TSQualifiedName";) {
      let { left, right } = object;
      previous = previous.object = {
        type: "MemberExpression",
        object: left,
        property: right,
        optional: false,
        computed: false,
        start: object.start,
        end: object.end,
      };
      object = left;
    }
  }
  return {
    type: "TSInterfaceHeritage",
    expression,
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypePredicate(pos) {
  return {
    type: "TSTypePredicate",
    parameterName: deserializeTSTypePredicateName(pos + 16),
    asserts: uint8[pos + 12] === 1,
    typeAnnotation: deserializeOptionBoxTSTypeAnnotation(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypePredicateName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSThisType(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypePredicateName`);
  }
}

function deserializeTSExternalModuleDeclaration(pos) {
  let start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    declare = deserializeBool(pos + 72),
    body = deserializeOptionBoxTSModuleBlock(pos + 64),
    node = {
      type: "TSModuleDeclaration",
      id: null,
      ...(body !== null && { body }),
      kind: "module",
      declare,
      global: false,
      start,
      end,
    };
  node.id = deserializeStringLiteral(pos + 16);
  return node;
}

function deserializeTSNamespaceDeclaration(pos) {
  let kind = deserializeTSNamespaceDeclarationKind(pos + 64),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    declare = deserializeBool(pos + 65),
    body = deserializeTSNamespaceDeclarationBody(pos + 48),
    node = {
      type: "TSModuleDeclaration",
      id: null,
      body,
      kind,
      declare,
      global: false,
      start,
      end,
    },
    id = deserializeBindingIdentifier(pos + 16);
  if (body.type === "TSModuleBlock") node.id = id;
  else {
    let innerId = body.id;
    if (innerId.type === "Identifier")
      node.id = {
        type: "TSQualifiedName",
        left: id,
        right: innerId,
        start: id.start,
        end: innerId.end,
      };
    else {
      // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with
      // `id` of this namespace on left, and previous `left` on right.
      node.id = innerId;
      let { start } = id;
      for (;;) {
        innerId.start = start;
        if (innerId.left.type === "Identifier") break;
        innerId = innerId.left;
      }
      let right = innerId.left;
      innerId.left = {
        type: "TSQualifiedName",
        left: id,
        right,
        start,
        end: right.end,
      };
    }
    body = body.body;
    node.body = body;
  }
  return node;
}

function deserializeTSNamespaceDeclarationKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "module";
    case 1:
      return "namespace";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSNamespaceDeclarationKind`);
  }
}

function deserializeTSNamespaceDeclarationBody(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSNamespaceDeclaration(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSModuleBlock(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSNamespaceDeclarationBody`);
  }
}

function deserializeTSGlobalDeclaration(pos) {
  return {
    type: "TSModuleDeclaration",
    id: {
      type: "Identifier",
      name: "global",
      start: deserializeI32(pos + 16),
      end: deserializeI32(pos + 20),
    },
    body: deserializeTSModuleBlock(pos + 24),
    kind: "global",
    declare: uint8[pos + 88] === 1,
    global: true,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSModuleBlock(pos) {
  let body;
  if (int32[(pos >> 2) + 6] === 0) body = deserializeVecStatement(pos + 40);
  else {
    body = deserializeVecDirective(pos + 16);
    body.push(...deserializeVecStatement(pos + 40));
  }
  return {
    type: "TSModuleBlock",
    body,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeLiteral(pos) {
  return {
    type: "TSTypeLiteral",
    members: deserializeVecTSSignature(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSInferType(pos) {
  return {
    type: "TSInferType",
    typeParameter: deserializeTSTypeParameter(int32[(pos >> 2) + 4]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeQuery(pos) {
  return {
    type: "TSTypeQuery",
    exprName: deserializeTSTypeQueryExprName(pos + 16),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeQueryExprName(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSQualifiedName(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeThisExpression(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeTSImportType(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSTypeQueryExprName`);
  }
}

function deserializeTSImportType(pos) {
  return {
    type: "TSImportType",
    source: deserializeStringLiteral(pos + 16),
    options: deserializeOptionBoxObjectExpression(pos + 64),
    qualifier: deserializeOptionTSImportTypeQualifier(pos + 72),
    typeArguments: deserializeOptionBoxTSTypeParameterInstantiation(pos + 88),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSImportTypeQualifier(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeIdentifierName(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeTSImportTypeQualifiedName(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSImportTypeQualifier`);
  }
}

function deserializeTSImportTypeQualifiedName(pos) {
  return {
    type: "TSQualifiedName",
    left: deserializeTSImportTypeQualifier(pos + 16),
    right: deserializeIdentifierName(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSFunctionType(pos) {
  let params = deserializeBoxFormalParameters(pos + 32),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  return {
    type: "TSFunctionType",
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 16),
    params,
    returnType: deserializeTSTypeAnnotation(int32[(pos >> 2) + 10]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSConstructorType(pos) {
  return {
    type: "TSConstructorType",
    abstract: uint8[pos + 40] === 1,
    typeParameters: deserializeOptionBoxTSTypeParameterDeclaration(pos + 16),
    params: deserializeFormalParameters(int32[(pos >> 2) + 6]),
    returnType: deserializeTSTypeAnnotation(int32[(pos >> 2) + 8]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSMappedType(pos) {
  let optional = deserializeOptionTSMappedTypeModifierOperator(pos + 96);
  optional === null && (optional = false);
  return {
    type: "TSMappedType",
    key: deserializeBindingIdentifier(pos + 16),
    constraint: deserializeTSType(pos + 48),
    nameType: deserializeOptionTSType(pos + 64),
    typeAnnotation: deserializeOptionTSType(pos + 80),
    optional,
    readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 97),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSMappedTypeModifierOperator(pos) {
  switch (uint8[pos]) {
    case 0:
      return true;
    case 1:
      return "+";
    case 2:
      return "-";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSMappedTypeModifierOperator`);
  }
}

function deserializeTSTemplateLiteralType(pos) {
  return {
    type: "TSTemplateLiteralType",
    quasis: deserializeVecTemplateElement(pos + 16),
    types: deserializeVecTSType(pos + 40),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSAsExpression(pos) {
  return {
    type: "TSAsExpression",
    expression: deserializeExpression(pos + 16),
    typeAnnotation: deserializeTSType(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSSatisfiesExpression(pos) {
  return {
    type: "TSSatisfiesExpression",
    expression: deserializeExpression(pos + 16),
    typeAnnotation: deserializeTSType(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSTypeAssertion(pos) {
  return {
    type: "TSTypeAssertion",
    typeAnnotation: deserializeTSType(pos + 16),
    expression: deserializeExpression(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSImportEqualsDeclaration(pos) {
  return {
    type: "TSImportEqualsDeclaration",
    id: deserializeBindingIdentifier(pos + 16),
    moduleReference: deserializeTSModuleReference(pos + 48),
    importKind: uint8[pos + 12] === 0 ? "value" : "type",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSModuleReference(pos) {
  switch (uint8[pos]) {
    case 0:
      return deserializeTSExternalModuleReference(int32[(pos >> 2) + 2]);
    case 1:
      return deserializeIdentifierReference(int32[(pos >> 2) + 2]);
    case 2:
      return deserializeTSQualifiedName(int32[(pos >> 2) + 2]);
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSModuleReference`);
  }
}

function deserializeTSExternalModuleReference(pos) {
  return {
    type: "TSExternalModuleReference",
    expression: deserializeStringLiteral(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSNonNullExpression(pos) {
  return {
    type: "TSNonNullExpression",
    expression: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeDecorator(pos) {
  return {
    type: "Decorator",
    expression: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSExportAssignment(pos) {
  return {
    type: "TSExportAssignment",
    expression: deserializeExpression(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSNamespaceExportDeclaration(pos) {
  return {
    type: "TSNamespaceExportDeclaration",
    id: deserializeIdentifierName(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeTSInstantiationExpression(pos) {
  return {
    type: "TSInstantiationExpression",
    expression: deserializeExpression(pos + 16),
    typeArguments: deserializeTSTypeParameterInstantiation(int32[(pos >> 2) + 8]),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSDocNullableType(pos) {
  return {
    type: "TSJSDocNullableType",
    typeAnnotation: deserializeTSType(pos + 16),
    postfix: uint8[pos + 12] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSDocNonNullableType(pos) {
  return {
    type: "TSJSDocNonNullableType",
    typeAnnotation: deserializeTSType(pos + 16),
    postfix: uint8[pos + 12] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeJSDocUnknownType(pos) {
  return {
    type: "TSJSDocUnknownType",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeModuleKind(pos) {
  switch (uint8[pos]) {
    case 0:
      return "script";
    case 1:
      return "module";
    case 3:
      return "commonjs";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ModuleKind`);
  }
}

function deserializeSpan(pos) {
  return {
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeNameSpan(pos) {
  return {
    value: deserializeStr(pos + 8),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeImportEntry(pos) {
  return {
    importName: deserializeImportImportName(pos + 32),
    localName: deserializeNameSpan(pos + 64),
    isType: uint8[pos + 88] === 1,
  };
}

function deserializeImportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      return {
        kind: "NamespaceObject",
        name: null,
        start: null,
        end: null,
      };
    case 2:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: "Default",
        name: null,
        start,
        end,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ImportImportName`);
  }
}

function deserializeExportEntry(pos) {
  return {
    moduleRequest: deserializeOptionNameSpan(pos + 16),
    importName: deserializeExportImportName(pos + 40),
    exportName: deserializeExportExportName(pos + 72),
    localName: deserializeExportLocalName(pos + 104),
    isType: uint8[pos + 136] === 1,
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeExportImportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      return {
        kind: "All",
        name: null,
        start: null,
        end: null,
      };
    case 2:
      return {
        kind: "AllButDefault",
        name: null,
        start: null,
        end: null,
      };
    case 3:
      return {
        kind: "None",
        name: null,
        start: null,
        end: null,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportImportName`);
  }
}

function deserializeExportExportName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      var { start, end } = deserializeSpan(pos + 8);
      return {
        kind: "Default",
        name: null,
        start,
        end,
      };
    case 2:
      return {
        kind: "None",
        name: null,
        start: null,
        end: null,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportExportName`);
  }
}

function deserializeExportLocalName(pos) {
  switch (uint8[pos]) {
    case 0:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Name",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 1:
      var nameSpan = deserializeNameSpan(pos + 8);
      return {
        kind: "Default",
        name: nameSpan.value,
        start: nameSpan.start,
        end: nameSpan.end,
      };
    case 2:
      return {
        kind: "None",
        name: null,
        start: null,
        end: null,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for ExportLocalName`);
  }
}

function deserializeDynamicImport(pos) {
  return {
    moduleRequest: deserializeSpan(pos + 8),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

const AssignmentOperator_TABLE = [
    "=",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "**=",
    "<<=",
    ">>=",
    ">>>=",
    "|=",
    "^=",
    "&=",
    "||=",
    "&&=",
    "??=",
  ],
  BinaryOperator_TABLE = [
    "==",
    "!=",
    "===",
    "!==",
    "<",
    "<=",
    ">",
    ">=",
    "+",
    "-",
    "*",
    "/",
    "%",
    "**",
    "<<",
    ">>",
    ">>>",
    "|",
    "^",
    "&",
    "in",
    "instanceof",
  ],
  LogicalOperator_TABLE = ["||", "&&", "??"],
  UnaryOperator_TABLE = ["+", "-", "!", "~", "typeof", "void", "delete"];

function deserializeComments(raw, src) {
  let raw32 = new Int32Array(raw.buffer, raw.byteOffset, raw.byteLength >> 2),
    len = raw.byteLength / 24,
    arr = Array(len);
  for (let i = 0, pos = 0; i < len; i++, pos += 24) {
    let pos32 = pos >> 2,
      start = raw32[pos32],
      end = raw32[pos32 + 1],
      kind = raw[pos + 17];
    kind === 1 || kind === 2
      ? (arr[i] = {
          type: "Block",
          value: src.slice(start + 2, end - 2),
          start,
          end,
        })
      : (arr[i] = {
          type: "Line",
          value: src.slice(start + (kind === 0 ? 2 : kind), end),
          start,
          end,
        });
  }
  return arr;
}

function deserializeRawTransferData(pos) {
  let pos32 = pos >> 2,
    commentsLen = int32[pos32 + 38],
    comments = null,
    rawComments = null,
    rawSourceText = null;
  if (commentsLen === 0) comments = [];
  else {
    let commentsPtr = int32[pos32 + 36];
    rawComments = uint8.slice(commentsPtr, commentsPtr + commentsLen * 24);
    rawSourceText = sourceText;
  }
  let program = deserializeProgram(pos),
    module =
      (int32[pos32 + 44] | int32[pos32 + 50] | int32[pos32 + 56] | int32[pos32 + 62]) === 0
        ? {
            hasModuleSyntax: uint8[pos + 264] === 1,
            staticImports: [],
            staticExports: [],
            dynamicImports: [],
            importMetas: [],
          }
        : deserializeEcmaScriptModule(pos + 168),
    errors = int32[pos32 + 70] === 0 ? [] : deserializeVecError(pos + 272);
  return {
    get program() {
      return program;
    },
    get module() {
      return module;
    },
    get comments() {
      if (comments === null) {
        comments = deserializeComments(rawComments, rawSourceText);
        rawComments = rawSourceText = null;
      }
      return comments;
    },
    get errors() {
      return errors;
    },
  };
}

function deserializeError(pos) {
  return {
    severity: ErrorSeverity_TABLE[uint8[pos + 72]],
    message: deserializeStr(pos),
    labels: deserializeVecErrorLabel(pos + 16),
    helpMessage: deserializeOptionStr(pos + 40),
    codeframe: deserializeStr(pos + 56),
  };
}

const ErrorSeverity_TABLE = ["Error", "Warning", "Advice"];

function deserializeErrorLabel(pos) {
  return {
    message: deserializeOptionStr(pos + 8),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeEcmaScriptModule(pos) {
  return {
    hasModuleSyntax: uint8[pos + 96] === 1,
    staticImports: deserializeVecStaticImport(pos),
    staticExports: deserializeVecStaticExport(pos + 24),
    dynamicImports: deserializeVecDynamicImport(pos + 48),
    importMetas: deserializeVecSpan(pos + 72),
  };
}

function deserializeStaticImport(pos) {
  return {
    moduleRequest: deserializeNameSpan(pos + 8),
    entries: deserializeVecImportEntry(pos + 32),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeStaticExport(pos) {
  return {
    entries: deserializeVecExportEntry(pos + 8),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
  };
}

function deserializeStr(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return "";
  pos = int32[pos32];
  let end = pos + len;
  if (pos >= sourceStartPos && end <= firstNonAsciiPos)
    return sourceText.substr(pos - sourceStartPos, len);
  // Use `utf8Slice` for strings longer than 64 bytes
  if (len > 64) return utf8Slice.call(uint8, pos, end);
  // Copy bytes into temp array.
  // If any byte is non-ASCII, use `utf8Slice`.
  let arr = stringDecodeArrays[len];
  for (let i = 0; i < len; i++) {
    let b = uint8[pos + i];
    if (b >= 128) return utf8Slice.call(uint8, pos, end);
    arr[i] = b;
  }
  // Call `fromCharCode` with temp array
  return fromCharCode.apply(null, arr);
}

function deserializeOptionHashbang(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeHashbang(pos);
}

function deserializeVecDirective(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeDirective(pos);
    pos += 80;
  }
  return arr;
}

function deserializeVecStatement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeStatement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeBoxIdentifierReference(pos) {
  return deserializeIdentifierReference(int32[pos >> 2]);
}

function deserializeBoxThisExpression(pos) {
  return deserializeThisExpression(int32[pos >> 2]);
}

function deserializeVecArrayExpressionElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeArrayExpressionElement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecObjectPropertyKind(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeObjectPropertyKind(pos);
    pos += 16;
  }
  return arr;
}

function deserializeBool(pos) {
  return uint8[pos] === 1;
}

function deserializeVecTemplateElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTemplateElement(pos);
    pos += 48;
  }
  return arr;
}

function deserializeVecExpression(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeExpression(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxTSTypeParameterInstantiation(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSTypeParameterInstantiation(int32[pos >> 2]);
}

function deserializeJSStr(pos) {
  if (uint8[pos + 12] === 0) return deserializeStr(pos);
  let buffer = uint8,
    pos32 = pos >> 2,
    len = int32[pos32 + 2];
  pos = int32[pos32];
  let end = pos + len,
    out = "";
  for (; pos < end;) {
    let first = buffer[pos++],
      codePoint;
    codePoint =
      first < 128
        ? first
        : first < 224
          ? ((first & 31) << 6) | (buffer[pos++] & 63)
          : first < 240
            ? ((first & 15) << 12) | ((buffer[pos++] & 63) << 6) | (buffer[pos++] & 63)
            : ((first & 7) << 18)
              | ((buffer[pos++] & 63) << 12)
              | ((buffer[pos++] & 63) << 6)
              | (buffer[pos++] & 63);
    // Unlike UTF-8 decoders, `fromCodePoint` preserves surrogate code points
    out += String.fromCodePoint(codePoint);
  }
  return out;
}

function deserializeOptionJSStr(pos) {
  return uint8[pos + 12] === 2 ? null : deserializeJSStr(pos);
}

function deserializeVecArgument(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeArgument(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionAssignmentTargetMaybeDefault(pos) {
  return uint8[pos] === 255 ? null : deserializeAssignmentTargetMaybeDefault(pos);
}

function deserializeVecOptionAssignmentTargetMaybeDefault(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeOptionAssignmentTargetMaybeDefault(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxAssignmentTargetRest(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeAssignmentTargetRest(int32[pos >> 2]);
}

function deserializeVecAssignmentTargetProperty(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeAssignmentTargetProperty(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionExpression(pos) {
  return uint8[pos] === 255 ? null : deserializeExpression(pos);
}

function deserializeVecVariableDeclarator(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeVariableDeclarator(pos);
    pos += 56;
  }
  return arr;
}

function deserializeOptionBoxTSTypeAnnotation(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSTypeAnnotation(int32[pos >> 2]);
}

function deserializeOptionStatement(pos) {
  return uint8[pos] === 255 ? null : deserializeStatement(pos);
}

function deserializeOptionForStatementInit(pos) {
  return uint8[pos] === 255 ? null : deserializeForStatementInit(pos);
}

function deserializeOptionLabelIdentifier(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeLabelIdentifier(pos);
}

function deserializeVecSwitchCase(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeSwitchCase(pos);
    pos += 56;
  }
  return arr;
}

function deserializeOptionBoxCatchClause(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeCatchClause(int32[pos >> 2]);
}

function deserializeOptionBoxBlockStatement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBlockStatement(int32[pos >> 2]);
}

function deserializeOptionCatchParameter(pos) {
  return uint8[pos + 16] === 255 ? null : deserializeCatchParameter(pos);
}

function deserializeVecBindingProperty(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeBindingProperty(pos);
    pos += 48;
  }
  return arr;
}

function deserializeOptionBoxBindingRestElement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeBindingRestElement(int32[pos >> 2]);
}

function deserializeOptionBindingPattern(pos) {
  return uint8[pos] === 255 ? null : deserializeBindingPattern(pos);
}

function deserializeVecOptionBindingPattern(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeOptionBindingPattern(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBindingIdentifier(pos) {
  return int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
    ? null
    : deserializeBindingIdentifier(pos);
}

function deserializeOptionBoxTSTypeParameterDeclaration(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSTypeParameterDeclaration(int32[pos >> 2]);
}

function deserializeOptionBoxTSThisParameter(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSThisParameter(int32[pos >> 2]);
}

function deserializeBoxFormalParameters(pos) {
  return deserializeFormalParameters(int32[pos >> 2]);
}

function deserializeOptionBoxFunctionBody(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeFunctionBody(int32[pos >> 2]);
}

function deserializeVecFormalParameter(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeFormalParameter(pos);
    pos += 72;
  }
  return arr;
}

function deserializeVecDecorator(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeDecorator(pos);
    pos += 32;
  }
  return arr;
}

function deserializeOptionBoxExpression(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeExpression(int32[pos >> 2]);
}

function deserializeVecClassElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeClassElement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionImportPhase(pos) {
  return uint8[pos] === 2 ? null : deserializeImportPhase(pos);
}

function deserializeVecImportDeclarationSpecifier(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeImportDeclarationSpecifier(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionVecImportDeclarationSpecifier(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeVecImportDeclarationSpecifier(pos);
}

function deserializeOptionBoxWithClause(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeWithClause(int32[pos >> 2]);
}

function deserializeVecImportAttribute(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeImportAttribute(pos);
    pos += 120;
  }
  return arr;
}

function deserializeVecExportSpecifier(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeExportSpecifier(pos);
    pos += 128;
  }
  return arr;
}

function deserializeOptionModuleExportName(pos) {
  return uint8[pos] === 255 ? null : deserializeModuleExportName(pos);
}

function deserializeOptionStr(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0 ? null : deserializeStr(pos);
}

function deserializeU8(pos) {
  return uint8[pos];
}

function deserializeBoxJSXOpeningElement(pos) {
  return deserializeJSXOpeningElement(int32[pos >> 2]);
}

function deserializeVecJSXChild(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeJSXChild(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxJSXClosingElement(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeJSXClosingElement(int32[pos >> 2]);
}

function deserializeVecJSXAttributeItem(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeJSXAttributeItem(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionJSXAttributeValue(pos) {
  return uint8[pos] === 255 ? null : deserializeJSXAttributeValue(pos);
}

function deserializeVecTSEnumMember(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSEnumMember(pos);
    pos += 48;
  }
  return arr;
}

function deserializeVecTSType(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSType(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecTSTupleElement(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSTupleElement(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionTSType(pos) {
  return uint8[pos] === 255 ? null : deserializeTSType(pos);
}

function deserializeVecTSTypeParameter(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSTypeParameter(pos);
    pos += 80;
  }
  return arr;
}

function deserializeVecTSInterfaceHeritage(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSInterfaceHeritage(pos);
    pos += 40;
  }
  return arr;
}

function deserializeVecTSSignature(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSSignature(pos);
    pos += 16;
  }
  return arr;
}

function deserializeOptionBoxTSModuleBlock(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeTSModuleBlock(int32[pos >> 2]);
}

function deserializeOptionBoxObjectExpression(pos) {
  return int32[pos >> 2] === 0 && int32[(pos >> 2) + 1] === 0
    ? null
    : deserializeObjectExpression(int32[pos >> 2]);
}

function deserializeOptionTSImportTypeQualifier(pos) {
  return uint8[pos] === 2 ? null : deserializeTSImportTypeQualifier(pos);
}

function deserializeOptionTSMappedTypeModifierOperator(pos) {
  return uint8[pos] === 255 ? null : deserializeTSMappedTypeModifierOperator(pos);
}

function deserializeI32(pos) {
  return int32[pos >> 2];
}

function deserializeOptionNameSpan(pos) {
  return int32[(pos >> 2) + 2] === 0 && int32[(pos >> 2) + 3] === 0
    ? null
    : deserializeNameSpan(pos);
}

function deserializeVecError(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeError(pos);
    pos += 80;
  }
  return arr;
}

function deserializeVecErrorLabel(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeErrorLabel(pos);
    pos += 24;
  }
  return arr;
}

function deserializeVecStaticImport(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeStaticImport(pos);
    pos += 56;
  }
  return arr;
}

function deserializeVecStaticExport(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeStaticExport(pos);
    pos += 32;
  }
  return arr;
}

function deserializeVecDynamicImport(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeDynamicImport(pos);
    pos += 16;
  }
  return arr;
}

function deserializeVecSpan(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeSpan(pos);
    pos += 8;
  }
  return arr;
}

function deserializeVecImportEntry(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeImportEntry(pos);
    pos += 96;
  }
  return arr;
}

function deserializeVecExportEntry(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeExportEntry(pos);
    pos += 144;
  }
  return arr;
}
