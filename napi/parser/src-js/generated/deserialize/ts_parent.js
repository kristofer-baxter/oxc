// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

let uint8,
  int32,
  float64,
  sourceText,
  sourceStartPos = 0,
  firstNonAsciiPos = 0,
  parent = null;

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
    program = (parent = {
      type: "Program",
      body: null,
      sourceType: deserializeModuleKind(pos + 137),
      hashbang: null,
      start,
      end,
      parent: null,
    });
  program.hashbang = deserializeOptionHashbang(pos + 56);
  int32[(pos >> 2) + 22 + 2] === 0
    ? (program.body = deserializeVecStatement(pos + 112))
    : (program.body = deserializeVecDirective(pos + 88)).push(
        ...deserializeVecStatement(pos + 112),
      );
  parent = null;
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
    decorators: [],
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    parent,
  };
}

function deserializeIdentifierReference(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    decorators: [],
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    parent,
  };
}

function deserializeBindingIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    decorators: [],
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    parent,
  };
}

function deserializeLabelIdentifier(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return {
    type: "Identifier",
    decorators: [],
    name:
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16),
    optional: false,
    typeAnnotation: null,
    start,
    end,
    parent,
  };
}

function deserializeThisExpression(pos) {
  return {
    type: "ThisExpression",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeArrayExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ArrayExpression",
      elements: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.elements = deserializeVecArrayExpressionElement(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ObjectExpression",
      properties: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.properties = deserializeVecObjectPropertyKind(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: PropertyKind_TABLE[uint8[pos + 12]],
      key: null,
      value: null,
      method: uint8[pos + 13] === 1,
      shorthand: uint8[pos + 14] === 1,
      computed: uint8[pos + 15] === 1,
      optional: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TemplateLiteral",
      quasis: null,
      expressions: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.expressions = deserializeVecExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTaggedTemplateExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TaggedTemplateExpression",
      tag: null,
      typeArguments: null,
      quasi: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.tag = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.quasi = deserializeTemplateLiteral(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTemplateElement(pos) {
  let tail = deserializeBool(pos + 12),
    start = deserializeI32(pos) - 1,
    end = deserializeI32(pos + 4) + 2 - tail;
  return {
    type: "TemplateElement",
    value: deserializeTemplateElementValue(pos + 16),
    tail,
    start,
    end,
    parent,
  };
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionJSStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: uint8[pos + 12] === 1,
      computed: true,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeStaticMemberExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: uint8[pos + 12] === 1,
      computed: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializePrivateFieldExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "MemberExpression",
      object: null,
      property: null,
      optional: uint8[pos + 12] === 1,
      computed: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.property = deserializePrivateIdentifier(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeCallExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "CallExpression",
      callee: null,
      typeArguments: null,
      arguments: null,
      optional: uint8[pos + 12] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.callee = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.arguments = deserializeVecArgument(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeNewExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "NewExpression",
      callee: null,
      typeArguments: null,
      arguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.callee = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.arguments = deserializeVecArgument(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeImportMeta(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "MetaProperty",
      meta: null,
      property: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    importStart = deserializeI32(pos),
    importIdent = {
      type: "Identifier",
      decorators: [],
      name: "import",
      optional: false,
      typeAnnotation: null,
      start: importStart,
      end: deserializeI32(pos + 4) === 0 ? 0 : importStart + 6,
      parent,
    },
    metaEnd = deserializeI32(pos + 4),
    metaIdent = {
      type: "Identifier",
      decorators: [],
      name: "meta",
      optional: false,
      typeAnnotation: null,
      start: metaEnd === 0 ? 0 : metaEnd - 4,
      end: metaEnd,
      parent,
    };
  node.meta = importIdent;
  node.property = metaIdent;
  parent = previousParent;
  return node;
}

function deserializeNewTarget(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "MetaProperty",
      meta: null,
      property: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    newStart = deserializeI32(pos),
    newIdent = {
      type: "Identifier",
      decorators: [],
      name: "new",
      optional: false,
      typeAnnotation: null,
      start: newStart,
      end: deserializeI32(pos + 4) === 0 ? 0 : newStart + 3,
      parent,
    },
    targetEnd = deserializeI32(pos + 4),
    targetIdent = {
      type: "Identifier",
      decorators: [],
      name: "target",
      optional: false,
      typeAnnotation: null,
      start: targetEnd === 0 ? 0 : targetEnd - 6,
      end: targetEnd,
      parent,
    };
  node.meta = newIdent;
  node.property = targetIdent;
  parent = previousParent;
  return node;
}

function deserializeSpreadElement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "SpreadElement",
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "UpdateExpression",
      operator: uint8[pos + 12] === 0 ? "++" : "--",
      prefix: uint8[pos + 13] === 1,
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeSimpleAssignmentTarget(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeUnaryExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "UnaryExpression",
      operator: UnaryOperator_TABLE[uint8[pos + 12]],
      argument: null,
      prefix: true,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBinaryExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "BinaryExpression",
      left: null,
      operator: BinaryOperator_TABLE[uint8[pos + 12]],
      right: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializePrivateInExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "BinaryExpression",
      left: null,
      operator: "in",
      right: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializePrivateIdentifier(pos + 16);
  node.right = deserializeExpression(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeLogicalExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "LogicalExpression",
      left: null,
      operator: LogicalOperator_TABLE[uint8[pos + 12]],
      right: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeConditionalExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ConditionalExpression",
      test: null,
      consequent: null,
      alternate: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeExpression(pos + 32);
  node.alternate = deserializeExpression(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeAssignmentExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "AssignmentExpression",
      operator: AssignmentOperator_TABLE[uint8[pos + 12]],
      left: null,
      right: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeAssignmentTarget(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ArrayPattern",
      decorators: [],
      elements: null,
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  parent = previousParent;
  return node;
}

function deserializeObjectAssignmentTarget(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ObjectPattern",
      decorators: [],
      properties: null,
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    properties = deserializeVecAssignmentTargetProperty(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetRest(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "RestElement",
      decorators: [],
      argument: null,
      optional: false,
      typeAnnotation: null,
      value: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeAssignmentTarget(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "AssignmentPattern",
      decorators: [],
      left: null,
      right: null,
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeAssignmentTarget(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
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
    previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: true,
      computed: false,
      optional: false,
      start,
      end,
      parent,
    }),
    key = deserializeIdentifierReference(pos + 16),
    value = {
      type: "Identifier",
      decorators: [],
      name: key.name,
      optional: false,
      typeAnnotation: null,
      start: key.start,
      end: key.end,
      parent,
    },
    init = deserializeOptionExpression(pos + 48);
  if (init !== null) {
    let left = value;
    value = {
      type: "AssignmentPattern",
      decorators: [],
      left,
      right: init,
      optional: false,
      typeAnnotation: null,
      start,
      end,
      parent,
    };
    left.parent = value;
    init.parent = value;
  }
  node.key = key;
  node.value = value;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: false,
      computed: uint8[pos + 12] === 1,
      optional: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeAssignmentTargetMaybeDefault(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSequenceExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "SequenceExpression",
      expressions: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expressions = deserializeVecExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeSuper(pos) {
  return {
    type: "Super",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeAwaitExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "AwaitExpression",
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeChainExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ChainExpression",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeChainElement(pos + 16);
  parent = previousParent;
  return node;
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
  {
    let previousParent = parent;
    node = parent = {
      type: "ParenthesizedExpression",
      expression: null,
      start: deserializeI32(pos),
      end: deserializeI32(pos + 4),
      parent,
    };
    node.expression = deserializeExpression(pos + 16);
    parent = previousParent;
  }
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
  let previousParent = parent,
    node = (parent = {
      type: "ExpressionStatement",
      expression: null,
      directive: deserializeStr(pos + 64),
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeStringLiteral(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeHashbang(pos) {
  return {
    type: "Hashbang",
    value: deserializeStr(pos + 16),
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeBlockStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "BlockStatement",
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.body = deserializeVecStatement(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "VariableDeclaration",
      kind: VariableDeclarationKind_TABLE[uint8[pos + 12]],
      declarations: null,
      declare: uint8[pos + 13] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.declarations = deserializeVecVariableDeclarator(pos + 16);
  parent = previousParent;
  return node;
}

const VariableDeclarationKind_TABLE = ["var", "let", "const", "using", "await using"];

function deserializeVariableDeclarator(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "VariableDeclarator",
      id: null,
      init: null,
      definite: uint8[pos + 12] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    pattern = deserializeBindingPattern(pos + 16);
  {
    let previousParent = parent;
    parent = pattern;
    let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
    if (typeAnnotation !== null) {
      pattern.typeAnnotation = typeAnnotation;
      pattern.end = typeAnnotation.end;
    }
    parent = previousParent;
  }
  node.id = pattern;
  node.init = deserializeOptionExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeEmptyStatement(pos) {
  return {
    type: "EmptyStatement",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeExpressionStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ExpressionStatement",
      expression: null,
      directive: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeIfStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "IfStatement",
      test: null,
      consequent: null,
      alternate: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeStatement(pos + 32);
  node.alternate = deserializeOptionStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeDoWhileStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "DoWhileStatement",
      body: null,
      test: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.body = deserializeStatement(pos + 16);
  node.test = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeWhileStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "WhileStatement",
      test: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.test = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeForStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ForStatement",
      init: null,
      test: null,
      update: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.init = deserializeOptionForStatementInit(pos + 16);
  node.test = deserializeOptionExpression(pos + 32);
  node.update = deserializeOptionExpression(pos + 48);
  node.body = deserializeStatement(pos + 64);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ForInStatement",
      left: null,
      right: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ForOfStatement",
      await: uint8[pos + 64] === 1,
      left: null,
      right: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeContinueStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ContinueStatement",
      label: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBreakStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "BreakStatement",
      label: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeReturnStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ReturnStatement",
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeOptionExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeWithStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "WithStatement",
      object: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.object = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSwitchStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "SwitchStatement",
      discriminant: null,
      cases: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.discriminant = deserializeExpression(pos + 16);
  node.cases = deserializeVecSwitchCase(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSwitchCase(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "SwitchCase",
      test: null,
      consequent: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.test = deserializeOptionExpression(pos + 16);
  node.consequent = deserializeVecStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeLabeledStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "LabeledStatement",
      label: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.label = deserializeLabelIdentifier(pos + 16);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeThrowStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ThrowStatement",
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTryStatement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TryStatement",
      block: null,
      handler: null,
      finalizer: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.block = deserializeBlockStatement(int32[(pos >> 2) + 4]);
  node.handler = deserializeOptionBoxCatchClause(pos + 24);
  node.finalizer = deserializeOptionBoxBlockStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeCatchClause(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "CatchClause",
      param: null,
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.param = deserializeOptionCatchParameter(pos + 16);
  node.body = deserializeBlockStatement(int32[(pos >> 2) + 14]);
  parent = previousParent;
  return node;
}

function deserializeCatchParameter(pos) {
  let previousParent = parent,
    pattern = deserializeBindingPattern(pos + 16);
  {
    parent = pattern;
    let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
    pattern.typeAnnotation = typeAnnotation;
    typeAnnotation !== null && (pattern.end = typeAnnotation.end);
    parent = previousParent;
  }
  return pattern;
}

function deserializeDebuggerStatement(pos) {
  return {
    type: "DebuggerStatement",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
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
  let previousParent = parent,
    node = (parent = {
      type: "AssignmentPattern",
      decorators: [],
      left: null,
      right: null,
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeBindingPattern(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeObjectPattern(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ObjectPattern",
      decorators: [],
      properties: null,
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    properties = deserializeVecBindingProperty(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  parent = previousParent;
  return node;
}

function deserializeBindingProperty(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "Property",
      kind: "init",
      key: null,
      value: null,
      method: false,
      shorthand: uint8[pos + 12] === 1,
      computed: uint8[pos + 13] === 1,
      optional: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeBindingPattern(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeArrayPattern(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ArrayPattern",
      decorators: [],
      elements: null,
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    elements = deserializeVecOptionBindingPattern(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  parent = previousParent;
  return node;
}

function deserializeBindingRestElement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "RestElement",
      decorators: [],
      argument: null,
      optional: false,
      typeAnnotation: null,
      value: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeBindingPattern(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeFunction(pos) {
  let previousParent = parent,
    node = (parent = {
      type: FunctionType_TABLE[uint8[pos + 88]],
      id: null,
      generator: uint8[pos + 89] === 1,
      async: uint8[pos + 90] === 1,
      declare: uint8[pos + 91] === 1,
      typeParameters: null,
      params: null,
      returnType: null,
      body: null,
      expression: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 64);
  {
    let thisParam = deserializeOptionBoxTSThisParameter(pos + 56);
    thisParam !== null && params.unshift(thisParam);
  }
  node.id = deserializeOptionBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 72);
  node.body = deserializeOptionBoxFunctionBody(pos + 80);
  parent = previousParent;
  return node;
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
    let start,
      end,
      previousParent = parent,
      rest = (parent = {
        type: "RestElement",
        decorators: [],
        argument: null,
        optional: false,
        typeAnnotation: null,
        value: null,
        start: (start = deserializeI32(pos + 40)),
        end: (end = deserializeI32(pos + 44)),
        parent: previousParent,
      });
    rest.argument = deserializeBindingPattern(pos + 56);
    rest.decorators = deserializeVecDecorator(pos + 16);
    rest.decorators.length !== 0 && (start = rest.decorators[0].start);
    rest.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 72);
    rest.typeAnnotation !== null && (end = rest.typeAnnotation.end);
    rest.start = start;
    rest.end = end;
    params.push(rest);
    parent = previousParent;
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let param,
    previousParent = parent,
    initializerFieldPos32 = (pos >> 2) + 16,
    hasInitializer = int32[initializerFieldPos32] !== 0 && int32[initializerFieldPos32 + 1] !== 0;
  {
    let accessibility = deserializeOptionTSAccessibility(pos + 13),
      readonly = deserializeBool(pos + 14),
      override = deserializeBool(pos + 15);
    if (accessibility === null && !readonly && !override) {
      let optional = deserializeBool(pos + 12);
      if (hasInitializer) {
        param = parent = {
          type: "AssignmentPattern",
          decorators: null,
          left: null,
          right: null,
          optional,
          typeAnnotation: null,
          start: deserializeI32(pos),
          end: deserializeI32(pos + 4),
          parent: previousParent,
        };
        param.decorators = deserializeVecDecorator(pos + 16);
        param.left = deserializeBindingPattern(pos + 40);
        param.left.decorators = [];
        param.left.optional = false;
        parent = param.left;
        let leftTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.left.typeAnnotation = leftTypeAnnotation;
        leftTypeAnnotation !== null && (param.left.end = leftTypeAnnotation.end);
        parent = param;
        param.right = deserializeOptionBoxExpression(pos + 64);
      } else {
        param = deserializeBindingPattern(pos + 40);
        param.parent = previousParent;
        parent = param;
        param.decorators = deserializeVecDecorator(pos + 16);
        param.optional = optional;
        let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.typeAnnotation = typeAnnotation;
        typeAnnotation === null
          ? optional && (param.end = deserializeI32(pos + 4))
          : (param.end = typeAnnotation.end);
        parent = previousParent;
      }
    } else {
      param = parent = {
        type: "TSParameterProperty",
        accessibility,
        decorators: null,
        override,
        parameter: null,
        readonly,
        static: false,
        start: deserializeI32(pos),
        end: deserializeI32(pos + 4),
        parent: previousParent,
      };
      param.decorators = deserializeVecDecorator(pos + 16);
      if (hasInitializer) {
        let pattern = deserializeBindingPattern(pos + 40),
          initializer = deserializeOptionBoxExpression(pos + 64),
          assignParam = (parent = {
            type: "AssignmentPattern",
            decorators: [],
            left: null,
            right: null,
            optional: false,
            typeAnnotation: null,
            start: pattern.start,
            end: initializer.end,
            parent: param,
          });
        assignParam.left = pattern;
        pattern.parent = assignParam;
        pattern.decorators = [];
        pattern.optional = false;
        parent = pattern;
        let patternTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        pattern.typeAnnotation = patternTypeAnnotation;
        patternTypeAnnotation !== null && (pattern.end = patternTypeAnnotation.end);
        parent = assignParam;
        assignParam.right = initializer;
        initializer !== null && (initializer.parent = assignParam);
        param.parameter = assignParam;
      } else {
        param.parameter = deserializeBindingPattern(pos + 40);
        param.parameter.decorators = [];
        let paramOptional = deserializeBool(pos + 12);
        param.parameter.optional = paramOptional;
        parent = param.parameter;
        let paramTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.parameter.typeAnnotation = paramTypeAnnotation;
        if (paramTypeAnnotation !== null) param.parameter.end = paramTypeAnnotation.end;
        else if (paramOptional) {
          let paramEnd = deserializeI32(pos + 4);
          param.parameter.end = paramEnd;
        }
        parent = param;
      }
    }
  }
  parent = previousParent;
  return param;
}

function deserializeFunctionBody(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "BlockStatement",
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    body;
  if (int32[(pos >> 2) + 6] === 0) body = deserializeVecStatement(pos + 40);
  else {
    body = deserializeVecDirective(pos + 16);
    body.push(...deserializeVecStatement(pos + 40));
  }
  node.body = body;
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ArrowFunctionExpression",
      expression: uint8[pos + 40] !== 64,
      async: uint8[pos + 56] === 1,
      typeParameters: null,
      params: null,
      returnType: null,
      body: null,
      id: null,
      generator: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos >> 2) + 6]);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  node.body = deserializeArrowFunctionBody(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeYieldExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "YieldExpression",
      delegate: uint8[pos + 12] === 1,
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeOptionExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeClass(pos) {
  let previousParent = parent,
    node = (parent = {
      type: uint8[pos + 136] === 0 ? "ClassDeclaration" : "ClassExpression",
      decorators: null,
      id: null,
      typeParameters: null,
      superClass: null,
      superTypeArguments: null,
      implements: null,
      body: null,
      abstract: uint8[pos + 137] === 1,
      declare: uint8[pos + 138] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    superClass = deserializeOptionExpression(pos + 80);
  node.decorators = deserializeVecDecorator(pos + 16);
  node.id = deserializeOptionBindingIdentifier(pos + 40);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 72);
  node.superClass = superClass;
  node.superTypeArguments =
    superClass === null
      ? null
      : deserializeOptionBoxTSTypeParameterInstantiation(pos + 80 + (pos + 16 - pos));
  node.implements = deserializeVecTSClassImplements(pos + 104);
  node.body = deserializeClassBody(int32[(pos >> 2) + 32]);
  parent = previousParent;
  return node;
}

function deserializeClassBody(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ClassBody",
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.body = deserializeVecClassElement(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: uint8[pos + 12] === 0 ? "MethodDefinition" : "TSAbstractMethodDefinition",
      decorators: null,
      key: null,
      value: null,
      kind: MethodDefinitionKind_TABLE[uint8[pos + 13]],
      computed: uint8[pos + 14] === 1,
      static: uint8[pos + 15] === 1,
      override: uint8[pos + 64] === 1,
      optional: uint8[pos + 65] === 1,
      accessibility: deserializeOptionTSAccessibility(pos + 66),
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.value = deserializeFunction(int32[(pos >> 2) + 14]);
  parent = previousParent;
  return node;
}

function deserializePropertyDefinition(pos) {
  let previousParent = parent,
    node = (parent = {
      type: uint8[pos + 12] === 0 ? "PropertyDefinition" : "TSAbstractPropertyDefinition",
      decorators: null,
      key: null,
      typeAnnotation: null,
      value: null,
      computed: uint8[pos + 13] === 1,
      static: uint8[pos + 14] === 1,
      declare: uint8[pos + 15] === 1,
      override: uint8[pos + 80] === 1,
      optional: uint8[pos + 81] === 1,
      definite: uint8[pos + 82] === 1,
      readonly: uint8[pos + 83] === 1,
      accessibility: deserializeOptionTSAccessibility(pos + 84),
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  node.value = deserializeOptionExpression(pos + 64);
  parent = previousParent;
  return node;
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
    parent,
  };
}

function deserializeStaticBlock(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "StaticBlock",
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.body = deserializeVecStatement(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeAccessorProperty(pos) {
  let previousParent = parent,
    node = (parent = {
      type: uint8[pos + 12] === 0 ? "AccessorProperty" : "TSAbstractAccessorProperty",
      decorators: null,
      key: null,
      typeAnnotation: null,
      value: null,
      computed: uint8[pos + 13] === 1,
      static: uint8[pos + 14] === 1,
      override: uint8[pos + 15] === 1,
      definite: uint8[pos + 80] === 1,
      accessibility: deserializeOptionTSAccessibility(pos + 81),
      declare: false,
      optional: false,
      readonly: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  node.value = deserializeOptionExpression(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeImportExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ImportExpression",
      source: null,
      options: null,
      phase: deserializeOptionImportPhase(pos + 12),
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.source = deserializeExpression(pos + 16);
  node.options = deserializeOptionExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeImportDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ImportDeclaration",
      specifiers: null,
      source: null,
      phase: deserializeOptionImportPhase(pos + 12),
      attributes: null,
      importKind: uint8[pos + 13] === 0 ? "value" : "type",
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    specifiers = deserializeOptionVecImportDeclarationSpecifier(pos + 16);
  specifiers === null && (specifiers = []);
  let withClause = deserializeOptionBoxWithClause(pos + 88);
  node.specifiers = specifiers;
  node.source = deserializeStringLiteral(pos + 40);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ImportSpecifier",
      imported: null,
      local: null,
      importKind: uint8[pos + 12] === 0 ? "value" : "type",
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.imported = deserializeModuleExportName(pos + 16);
  node.local = deserializeBindingIdentifier(pos + 72);
  parent = previousParent;
  return node;
}

function deserializeImportDefaultSpecifier(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ImportDefaultSpecifier",
      local: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.local = deserializeBindingIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeImportNamespaceSpecifier(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ImportNamespaceSpecifier",
      local: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.local = deserializeBindingIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 16) };
}

function deserializeImportAttribute(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ImportAttribute",
      key: null,
      value: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.key = deserializeImportAttributeKey(pos + 16);
  node.value = deserializeStringLiteral(pos + 72);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: [],
      source: null,
      exportKind: null,
      attributes: [],
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    declaration = deserializeDeclaration(pos + 16);
  node.declaration = declaration;
  node.exportKind =
    declaration.declare === true
    || declaration.type === "TSTypeAliasDeclaration"
    || declaration.type === "TSInterfaceDeclaration"
      ? "type"
      : "value";
  parent = previousParent;
  return node;
}

function deserializeExportNamedDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      exportKind: uint8[pos + 12] === 0 ? "value" : "type",
      attributes: [],
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.specifiers = deserializeVecExportSpecifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeExportFromDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ExportNamedDeclaration",
      declaration: null,
      specifiers: null,
      source: null,
      exportKind: uint8[pos + 12] === 0 ? "value" : "type",
      attributes: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    withClause = deserializeOptionBoxWithClause(pos + 88);
  node.specifiers = deserializeVecExportSpecifier(pos + 16);
  node.source = deserializeStringLiteral(pos + 40);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportDefaultDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ExportDefaultDeclaration",
      declaration: null,
      exportKind: "value",
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.declaration = deserializeExportDefaultDeclarationKind(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeExportAllDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ExportAllDeclaration",
      exported: null,
      source: null,
      attributes: null,
      exportKind: uint8[pos + 12] === 0 ? "value" : "type",
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    withClause = deserializeOptionBoxWithClause(pos + 120);
  node.exported = deserializeOptionModuleExportName(pos + 16);
  node.source = deserializeStringLiteral(pos + 72);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportSpecifier(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "ExportSpecifier",
      local: null,
      exported: null,
      exportKind: uint8[pos + 12] === 0 ? "value" : "type",
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.local = deserializeModuleExportName(pos + 16);
  node.exported = deserializeModuleExportName(pos + 72);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "V8IntrinsicExpression",
      name: null,
      arguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.name = deserializeIdentifierName(pos + 16);
  node.arguments = deserializeVecArgument(pos + 48);
  parent = previousParent;
  return node;
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
    parent,
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
    parent,
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
    parent,
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
    parent,
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
    parent,
  };
}

function deserializeRegExpLiteral(pos) {
  let start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    previousParent = parent,
    node = (parent = {
      type: "Literal",
      value: null,
      raw:
        int32[(pos >> 2) + 12] === 0 && int32[(pos >> 2) + 13] === 0
          ? null
          : sourceText.slice(start, end),
      regex: null,
      start,
      end,
      parent,
    }),
    regex = deserializeRegExp(pos + 16),
    value = null;
  try {
    value = new RegExp(regex.pattern, regex.flags);
  } catch {}
  node.value = value;
  node.regex = regex;
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "JSXElement",
      openingElement: null,
      children: null,
      closingElement: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    closingElement = deserializeOptionBoxJSXClosingElement(pos + 48),
    openingElement = deserializeBoxJSXOpeningElement(pos + 16);
  closingElement === null && (openingElement.selfClosing = true);
  node.openingElement = openingElement;
  node.children = deserializeVecJSXChild(pos + 24);
  node.closingElement = closingElement;
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningElement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXOpeningElement",
      name: null,
      typeArguments: null,
      attributes: null,
      selfClosing: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.name = deserializeJSXElementName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.attributes = deserializeVecJSXAttributeItem(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeJSXClosingElement(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXClosingElement",
      name: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.name = deserializeJSXElementName(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSXFragment(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXFragment",
      openingFragment: null,
      children: null,
      closingFragment: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.openingFragment = deserializeJSXOpeningFragment(pos + 16);
  node.children = deserializeVecJSXChild(pos + 32);
  node.closingFragment = deserializeJSXClosingFragment(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningFragment(pos) {
  return {
    type: "JSXOpeningFragment",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeJSXClosingFragment(pos) {
  return {
    type: "JSXClosingFragment",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
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
        parent,
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
        parent,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXNamespacedName",
      namespace: null,
      name: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.namespace = deserializeJSXIdentifier(pos + 16);
  node.name = deserializeJSXIdentifier(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeJSXMemberExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXMemberExpression",
      object: null,
      property: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.object = deserializeJSXMemberExpressionObject(pos + 16);
  node.property = deserializeJSXIdentifier(pos + 32);
  parent = previousParent;
  return node;
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
        parent,
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
        parent,
      };
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXExpressionContainer",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeJSXExpression(pos + 16);
  parent = previousParent;
  return node;
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
    parent,
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
  let previousParent = parent,
    node = (parent = {
      type: "JSXAttribute",
      name: null,
      value: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.name = deserializeJSXAttributeName(pos + 16);
  node.value = deserializeOptionJSXAttributeValue(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeJSXSpreadAttribute(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "JSXSpreadAttribute",
      argument: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
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
    parent,
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
  let previousParent = parent,
    node = (parent = {
      type: "JSXSpreadChild",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
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
    parent,
  };
}

function deserializeTSThisParameter(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: [],
      name: "this",
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSEnumDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSEnumDeclaration",
      id: null,
      body: null,
      const: uint8[pos + 12] === 1,
      declare: uint8[pos + 13] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.body = deserializeTSEnumBody(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSEnumBody(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSEnumBody",
      members: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.members = deserializeVecTSEnumMember(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSEnumMember(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSEnumMember",
      id: null,
      initializer: null,
      computed: deserializeU8(pos + 16) > 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = deserializeTSEnumMemberName(pos + 16);
  node.initializer = deserializeOptionExpression(pos + 32);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeAnnotation",
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSLiteralType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSLiteralType",
      literal: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.literal = deserializeTSLiteral(pos + 16);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSConditionalType",
      checkType: null,
      extendsType: null,
      trueType: null,
      falseType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.checkType = deserializeTSType(pos + 16);
  node.extendsType = deserializeTSType(pos + 32);
  node.trueType = deserializeTSType(pos + 48);
  node.falseType = deserializeTSType(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeTSUnionType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSUnionType",
      types: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.types = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSIntersectionType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSIntersectionType",
      types: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.types = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSParenthesizedType(pos) {
  let node;
  {
    let previousParent = parent;
    node = parent = {
      type: "TSParenthesizedType",
      typeAnnotation: null,
      start: deserializeI32(pos),
      end: deserializeI32(pos + 4),
      parent,
    };
    node.typeAnnotation = deserializeTSType(pos + 16);
    parent = previousParent;
  }
  return node;
}

function deserializeTSTypeOperator(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeOperator",
      operator: TSTypeOperatorOperator_TABLE[uint8[pos + 12]],
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

const TSTypeOperatorOperator_TABLE = ["keyof", "unique", "readonly"];

function deserializeTSArrayType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSArrayType",
      elementType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.elementType = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSIndexedAccessType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSIndexedAccessType",
      objectType: null,
      indexType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.objectType = deserializeTSType(pos + 16);
  node.indexType = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTupleType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTupleType",
      elementTypes: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.elementTypes = deserializeVecTSTupleElement(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNamedTupleMember(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSNamedTupleMember",
      label: null,
      elementType: null,
      optional: uint8[pos + 12] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.label = deserializeIdentifierName(pos + 16);
  node.elementType = deserializeTSTupleElement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSOptionalType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSOptionalType",
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSRestType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSRestType",
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
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
    parent,
  };
}

function deserializeTSStringKeyword(pos) {
  return {
    type: "TSStringKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSBooleanKeyword(pos) {
  return {
    type: "TSBooleanKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSNumberKeyword(pos) {
  return {
    type: "TSNumberKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSNeverKeyword(pos) {
  return {
    type: "TSNeverKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSIntrinsicKeyword(pos) {
  return {
    type: "TSIntrinsicKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSUnknownKeyword(pos) {
  return {
    type: "TSUnknownKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSNullKeyword(pos) {
  return {
    type: "TSNullKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSUndefinedKeyword(pos) {
  return {
    type: "TSUndefinedKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSVoidKeyword(pos) {
  return {
    type: "TSVoidKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSSymbolKeyword(pos) {
  return {
    type: "TSSymbolKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSThisType(pos) {
  return {
    type: "TSThisType",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSObjectKeyword(pos) {
  return {
    type: "TSObjectKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSBigIntKeyword(pos) {
  return {
    type: "TSBigIntKeyword",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
  };
}

function deserializeTSTypeReference(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeReference",
      typeName: null,
      typeArguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeName = deserializeTSTypeName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeTSTypeName(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterInstantiation(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeParameterInstantiation",
      params: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.params = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameter(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeParameter",
      name: null,
      constraint: null,
      default: null,
      in: uint8[pos + 12] === 1,
      out: uint8[pos + 13] === 1,
      const: uint8[pos + 14] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.name = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeOptionTSType(pos + 48);
  node.default = deserializeOptionTSType(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeParameterDeclaration",
      params: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.params = deserializeVecTSTypeParameter(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAliasDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeAliasDeclaration",
      id: null,
      typeParameters: null,
      typeAnnotation: null,
      declare: uint8[pos + 72] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.typeAnnotation = deserializeTSType(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeTSAccessibility(pos) {
  switch (uint8[pos]) {
    case 0:
      return "private";
    case 1:
      return "protected";
    case 2:
      return "public";
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for TSAccessibility`);
  }
}

function deserializeTSClassImplements(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSClassImplements",
      expression: null,
      typeArguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    expression = deserializeTSTypeName(pos + 16);
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
        parent,
      });
    right.parent = previous;
    for (;;) {
      if (object.type !== "TSQualifiedName") {
        object.parent = previous;
        break;
      }
      let { left, right } = object;
      previous = previous.object = {
        type: "MemberExpression",
        object: left,
        property: right,
        optional: false,
        computed: false,
        start: object.start,
        end: object.end,
        parent: previous,
      };
      right.parent = previous;
      object = left;
    }
  }
  node.expression = expression;
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSInterfaceDeclaration",
      id: null,
      typeParameters: null,
      extends: null,
      body: null,
      declare: uint8[pos + 88] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.extends = deserializeVecTSInterfaceHeritage(pos + 56);
  node.body = deserializeTSInterfaceBody(int32[(pos >> 2) + 20]);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceBody(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSInterfaceBody",
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.body = deserializeVecTSSignature(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSPropertySignature(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSPropertySignature",
      computed: uint8[pos + 12] === 1,
      optional: uint8[pos + 13] === 1,
      readonly: uint8[pos + 14] === 1,
      key: null,
      typeAnnotation: null,
      accessibility: null,
      static: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.key = deserializePropertyKey(pos + 16);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSIndexSignature",
      parameters: null,
      typeAnnotation: null,
      readonly: uint8[pos + 12] === 1,
      static: uint8[pos + 13] === 1,
      accessibility: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.parameters = [deserializeTSIndexSignatureName(pos + 16)];
  node.typeAnnotation = deserializeTSTypeAnnotation(int32[(pos >> 2) + 14]);
  parent = previousParent;
  return node;
}

function deserializeTSCallSignatureDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSCallSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 32),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 40);
  parent = previousParent;
  return node;
}

const TSMethodSignatureKind_TABLE = ["method", "get", "set"];

function deserializeTSMethodSignature(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSMethodSignature",
      key: null,
      computed: uint8[pos + 64] === 1,
      optional: uint8[pos + 65] === 1,
      kind: TSMethodSignatureKind_TABLE[uint8[pos + 66]],
      typeParameters: null,
      params: null,
      returnType: null,
      accessibility: null,
      readonly: false,
      static: false,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 48),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 40);
  thisParam !== null && params.unshift(thisParam);
  node.key = deserializePropertyKey(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 32);
  node.params = params;
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeTSConstructSignatureDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSConstructSignatureDeclaration",
      typeParameters: null,
      params: null,
      returnType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos >> 2) + 6]);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSIndexSignatureName(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "Identifier",
      decorators: [],
      name: deserializeStr(pos + 16),
      optional: false,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSTypeAnnotation(int32[(pos >> 2) + 8]);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceHeritage(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSInterfaceHeritage",
      expression: null,
      typeArguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    expression = deserializeTSTypeName(pos + 16);
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
        parent,
      });
    right.parent = previous;
    for (;;) {
      if (object.type !== "TSQualifiedName") {
        object.parent = previous;
        break;
      }
      let { left, right } = object;
      previous = previous.object = {
        type: "MemberExpression",
        object: left,
        property: right,
        optional: false,
        computed: false,
        start: object.start,
        end: object.end,
        parent: previous,
      };
      right.parent = previous;
      object = left;
    }
  }
  node.expression = expression;
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypePredicate(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypePredicate",
      parameterName: null,
      asserts: uint8[pos + 12] === 1,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.parameterName = deserializeTSTypePredicateName(pos + 16);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
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
    previousParent = parent,
    body = deserializeOptionBoxTSModuleBlock(pos + 64),
    node = (parent = {
      type: "TSModuleDeclaration",
      id: null,
      ...(body !== null && { body }),
      kind: "module",
      declare,
      global: false,
      start,
      end,
      parent,
    });
  node.id = deserializeStringLiteral(pos + 16);
  body !== null && (body.parent = node);
  parent = previousParent;
  return node;
}

function deserializeTSNamespaceDeclaration(pos) {
  let kind = deserializeTSNamespaceDeclarationKind(pos + 64),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    declare = deserializeBool(pos + 65),
    previousParent = parent,
    body = deserializeTSNamespaceDeclarationBody(pos + 48),
    node = (parent = {
      type: "TSModuleDeclaration",
      id: null,
      body,
      kind,
      declare,
      global: false,
      start,
      end,
      parent,
    }),
    id = deserializeBindingIdentifier(pos + 16);
  if (body.type === "TSModuleBlock") {
    node.id = id;
    body.parent = node;
  } else {
    let innerId = body.id;
    if (innerId.type === "Identifier") {
      let outerId =
        (node.id =
        parent =
          {
            type: "TSQualifiedName",
            left: id,
            right: innerId,
            start: id.start,
            end: innerId.end,
            parent: node,
          });
      id.parent = innerId.parent = outerId;
    } else {
      // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with
      // `id` of this namespace on left, and previous `left` on right.
      node.id = innerId;
      innerId.parent = node;
      let { start } = id;
      for (;;) {
        innerId.start = start;
        if (innerId.left.type === "Identifier") break;
        innerId = innerId.left;
      }
      let right = innerId.left;
      id.parent =
        right.parent =
        innerId.left =
          {
            type: "TSQualifiedName",
            left: id,
            right,
            start,
            end: right.end,
            parent: innerId,
          };
    }
    body = body.body;
    node.body = body;
    body.parent = node;
  }
  parent = previousParent;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSModuleDeclaration",
      id: null,
      body: null,
      kind: "global",
      declare: uint8[pos + 88] === 1,
      global: true,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = {
    type: "Identifier",
    decorators: [],
    name: "global",
    optional: false,
    typeAnnotation: null,
    start: deserializeI32(pos + 16),
    end: deserializeI32(pos + 20),
    parent,
  };
  node.body = deserializeTSModuleBlock(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSModuleBlock(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSModuleBlock",
      body: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    body;
  if (int32[(pos >> 2) + 6] === 0) body = deserializeVecStatement(pos + 40);
  else {
    body = deserializeVecDirective(pos + 16);
    body.push(...deserializeVecStatement(pos + 40));
  }
  node.body = body;
  parent = previousParent;
  return node;
}

function deserializeTSTypeLiteral(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeLiteral",
      members: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.members = deserializeVecTSSignature(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSInferType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSInferType",
      typeParameter: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeParameter = deserializeTSTypeParameter(int32[(pos >> 2) + 4]);
  parent = previousParent;
  return node;
}

function deserializeTSTypeQuery(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeQuery",
      exprName: null,
      typeArguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.exprName = deserializeTSTypeQueryExprName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSImportType",
      source: null,
      options: null,
      qualifier: null,
      typeArguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.source = deserializeStringLiteral(pos + 16);
  node.options = deserializeOptionBoxObjectExpression(pos + 64);
  node.qualifier = deserializeOptionTSImportTypeQualifier(pos + 72);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 88);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSQualifiedName",
      left: null,
      right: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.left = deserializeTSImportTypeQualifier(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSFunctionType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSFunctionType",
      typeParameters: null,
      params: null,
      returnType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    params = deserializeBoxFormalParameters(pos + 32),
    thisParam = deserializeOptionBoxTSThisParameter(pos + 24);
  thisParam !== null && params.unshift(thisParam);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = params;
  node.returnType = deserializeTSTypeAnnotation(int32[(pos >> 2) + 10]);
  parent = previousParent;
  return node;
}

function deserializeTSConstructorType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSConstructorType",
      abstract: uint8[pos + 40] === 1,
      typeParameters: null,
      params: null,
      returnType: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos >> 2) + 6]);
  node.returnType = deserializeTSTypeAnnotation(int32[(pos >> 2) + 8]);
  parent = previousParent;
  return node;
}

function deserializeTSMappedType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSMappedType",
      key: null,
      constraint: null,
      nameType: null,
      typeAnnotation: null,
      optional: null,
      readonly: deserializeOptionTSMappedTypeModifierOperator(pos + 97),
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    }),
    optional = deserializeOptionTSMappedTypeModifierOperator(pos + 96);
  optional === null && (optional = false);
  node.key = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeTSType(pos + 48);
  node.nameType = deserializeOptionTSType(pos + 64);
  node.typeAnnotation = deserializeOptionTSType(pos + 80);
  node.optional = optional;
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSTemplateLiteralType",
      quasis: null,
      types: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.types = deserializeVecTSType(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTSAsExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSAsExpression",
      expression: null,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSSatisfiesExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSSatisfiesExpression",
      expression: null,
      typeAnnotation: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAssertion(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSTypeAssertion",
      typeAnnotation: null,
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  node.expression = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSImportEqualsDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSImportEqualsDeclaration",
      id: null,
      moduleReference: null,
      importKind: uint8[pos + 12] === 0 ? "value" : "type",
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = deserializeBindingIdentifier(pos + 16);
  node.moduleReference = deserializeTSModuleReference(pos + 48);
  parent = previousParent;
  return node;
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
  let previousParent = parent,
    node = (parent = {
      type: "TSExternalModuleReference",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeStringLiteral(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNonNullExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSNonNullExpression",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeDecorator(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "Decorator",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSExportAssignment(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSExportAssignment",
      expression: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSNamespaceExportDeclaration",
      id: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.id = deserializeIdentifierName(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSInstantiationExpression(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSInstantiationExpression",
      expression: null,
      typeArguments: null,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.expression = deserializeExpression(pos + 16);
  node.typeArguments = deserializeTSTypeParameterInstantiation(int32[(pos >> 2) + 8]);
  parent = previousParent;
  return node;
}

function deserializeJSDocNullableType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSJSDocNullableType",
      typeAnnotation: null,
      postfix: uint8[pos + 12] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSDocNonNullableType(pos) {
  let previousParent = parent,
    node = (parent = {
      type: "TSJSDocNonNullableType",
      typeAnnotation: null,
      postfix: uint8[pos + 12] === 1,
      start: int32[pos >> 2],
      end: int32[(pos >> 2) + 1],
      parent,
    });
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSDocUnknownType(pos) {
  return {
    type: "TSJSDocUnknownType",
    start: int32[pos >> 2],
    end: int32[(pos >> 2) + 1],
    parent,
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

function deserializeOptionTSAccessibility(pos) {
  return uint8[pos] === 255 ? null : deserializeTSAccessibility(pos);
}

function deserializeVecTSClassImplements(pos) {
  let pos32 = pos >> 2,
    len = int32[pos32 + 2];
  if (len === 0) return [];
  pos = int32[pos32];
  let arr = Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = deserializeTSClassImplements(pos);
    pos += 40;
  }
  return arr;
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
