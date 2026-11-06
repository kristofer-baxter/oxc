// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

import { tokens, initTokens } from "../plugins/tokens.js";
import { comments, initComments } from "../plugins/comments.js";
import { getNodeLoc } from "../plugins/location.js";

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

const NodeProto = Object.create(Object.prototype, {
  loc: {
    // Note: Not configurable
    get() {
      return getNodeLoc(this);
    },
    enumerable: true,
  },
});

export function deserializeProgramOnly(buffer, sourceText, sourceStartPos, sourceByteLen) {
  return deserializeWith(buffer, sourceText, sourceStartPos, sourceByteLen, deserializeProgram);
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
  let node$0,
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    program = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Program"),
      (node$0.body = null),
      (node$0.sourceType = deserializeModuleKind(pos + 137)),
      Object.defineProperty(node$0, "comments", {
        get() {
          comments === null && initComments();
          return comments;
        },
        enumerable: true,
        configurable: true,
      }),
      Object.defineProperty(node$0, "tokens", {
        get() {
          tokens === null && initTokens();
          return tokens;
        },
        enumerable: true,
        configurable: true,
      }),
      (node$0.start = start),
      (node$0.end = end),
      (node$0.range = [start, end]),
      (node$0.parent = null),
      node$0));
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
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Identifier"),
    (node$0.decorators = []),
    (node$0.name =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16)),
    (node$0.optional = false),
    (node$0.typeAnnotation = null),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeIdentifierReference(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Identifier"),
    (node$0.decorators = []),
    (node$0.name =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16)),
    (node$0.optional = false),
    (node$0.typeAnnotation = null),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeBindingIdentifier(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Identifier"),
    (node$0.decorators = []),
    (node$0.name =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16)),
    (node$0.optional = false),
    (node$0.typeAnnotation = null),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeLabelIdentifier(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Identifier"),
    (node$0.decorators = []),
    (node$0.name =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16)),
    (node$0.optional = false),
    (node$0.typeAnnotation = null),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeThisExpression(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "ThisExpression"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeArrayExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ArrayExpression"),
      (node$0.elements = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ObjectExpression"),
      (node$0.properties = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Property"),
      (node$0.kind = PropertyKind_TABLE[uint8[pos + 12]]),
      (node$0.key = null),
      (node$0.value = null),
      (node$0.method = uint8[pos + 13] === 1),
      (node$0.shorthand = uint8[pos + 14] === 1),
      (node$0.computed = uint8[pos + 15] === 1),
      (node$0.optional = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TemplateLiteral"),
      (node$0.quasis = null),
      (node$0.expressions = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.expressions = deserializeVecExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTaggedTemplateExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TaggedTemplateExpression"),
      (node$0.tag = null),
      (node$0.typeArguments = null),
      (node$0.quasi = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.tag = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.quasi = deserializeTemplateLiteral(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTemplateElement(pos) {
  let node$0,
    tail = deserializeBool(pos + 12),
    start = deserializeI32(pos) - 1,
    end = deserializeI32(pos + 4) + 2 - tail,
    value = deserializeTemplateElementValue(pos + 16);
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TemplateElement"),
    (node$0.value = value),
    (node$0.tail = tail),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTemplateElementValue(pos) {
  return {
    raw: deserializeStr(pos),
    cooked: deserializeOptionJSStr(pos + 16),
  };
}

function deserializeComputedMemberExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "MemberExpression"),
      (node$0.object = null),
      (node$0.property = null),
      (node$0.optional = uint8[pos + 12] === 1),
      (node$0.computed = true),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeStaticMemberExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "MemberExpression"),
      (node$0.object = null),
      (node$0.property = null),
      (node$0.optional = uint8[pos + 12] === 1),
      (node$0.computed = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.object = deserializeExpression(pos + 16);
  node.property = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializePrivateFieldExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "MemberExpression"),
      (node$0.object = null),
      (node$0.property = null),
      (node$0.optional = uint8[pos + 12] === 1),
      (node$0.computed = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.object = deserializeExpression(pos + 16);
  node.property = deserializePrivateIdentifier(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeCallExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "CallExpression"),
      (node$0.callee = null),
      (node$0.typeArguments = null),
      (node$0.arguments = null),
      (node$0.optional = uint8[pos + 12] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.callee = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.arguments = deserializeVecArgument(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeNewExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "NewExpression"),
      (node$0.callee = null),
      (node$0.typeArguments = null),
      (node$0.arguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.callee = deserializeExpression(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.arguments = deserializeVecArgument(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeImportMeta(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "MetaProperty"),
      (node$0.meta = null),
      (node$0.property = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    importStart = deserializeI32(pos),
    importEnd = deserializeI32(pos + 4) === 0 ? 0 : importStart + 6,
    importIdent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = "import"),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = importStart),
      (node$0.end = importEnd),
      (node$0.range = [importStart, importEnd]),
      (node$0.parent = parent),
      node$0),
    metaEnd = deserializeI32(pos + 4),
    metaStart = metaEnd === 0 ? 0 : metaEnd - 4,
    metaIdent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = "meta"),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = metaStart),
      (node$0.end = metaEnd),
      (node$0.range = [metaStart, metaEnd]),
      (node$0.parent = parent),
      node$0);
  node.meta = importIdent;
  node.property = metaIdent;
  parent = previousParent;
  return node;
}

function deserializeNewTarget(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "MetaProperty"),
      (node$0.meta = null),
      (node$0.property = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    newStart = deserializeI32(pos),
    newEnd = deserializeI32(pos + 4) === 0 ? 0 : newStart + 3,
    newIdent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = "new"),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = newStart),
      (node$0.end = newEnd),
      (node$0.range = [newStart, newEnd]),
      (node$0.parent = parent),
      node$0),
    targetEnd = deserializeI32(pos + 4),
    targetStart = targetEnd === 0 ? 0 : targetEnd - 6,
    targetIdent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = "target"),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = targetStart),
      (node$0.end = targetEnd),
      (node$0.range = [targetStart, targetEnd]),
      (node$0.parent = parent),
      node$0);
  node.meta = newIdent;
  node.property = targetIdent;
  parent = previousParent;
  return node;
}

function deserializeSpreadElement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "SpreadElement"),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "UpdateExpression"),
      (node$0.operator = uint8[pos + 12] === 0 ? "++" : "--"),
      (node$0.prefix = uint8[pos + 13] === 1),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeSimpleAssignmentTarget(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeUnaryExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "UnaryExpression"),
      (node$0.operator = UnaryOperator_TABLE[uint8[pos + 12]]),
      (node$0.argument = null),
      (node$0.prefix = true),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBinaryExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "BinaryExpression"),
      (node$0.left = null),
      (node$0.operator = BinaryOperator_TABLE[uint8[pos + 12]]),
      (node$0.right = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializePrivateInExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "BinaryExpression"),
      (node$0.left = null),
      (node$0.operator = "in"),
      (node$0.right = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializePrivateIdentifier(pos + 16);
  node.right = deserializeExpression(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeLogicalExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "LogicalExpression"),
      (node$0.left = null),
      (node$0.operator = LogicalOperator_TABLE[uint8[pos + 12]]),
      (node$0.right = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializeExpression(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeConditionalExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ConditionalExpression"),
      (node$0.test = null),
      (node$0.consequent = null),
      (node$0.alternate = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeExpression(pos + 32);
  node.alternate = deserializeExpression(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeAssignmentExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "AssignmentExpression"),
      (node$0.operator = AssignmentOperator_TABLE[uint8[pos + 12]]),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ArrayPattern"),
      (node$0.decorators = []),
      (node$0.elements = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    elements = deserializeVecOptionAssignmentTargetMaybeDefault(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  parent = previousParent;
  return node;
}

function deserializeObjectAssignmentTarget(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ObjectPattern"),
      (node$0.decorators = []),
      (node$0.properties = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    properties = deserializeVecAssignmentTargetProperty(pos + 16),
    rest = deserializeOptionBoxAssignmentTargetRest(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetRest(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "RestElement"),
      (node$0.decorators = []),
      (node$0.argument = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.value = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "AssignmentPattern"),
      (node$0.decorators = []),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Property"),
      (node$0.kind = "init"),
      (node$0.key = null),
      (node$0.value = null),
      (node$0.method = false),
      (node$0.shorthand = true),
      (node$0.computed = false),
      (node$0.optional = false),
      (node$0.start = start),
      (node$0.end = end),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    key = deserializeIdentifierReference(pos + 16),
    keyStart,
    keyEnd,
    value =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = key.name),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = keyStart = key.start),
      (node$0.end = keyEnd = key.end),
      (node$0.range = [keyStart, keyEnd]),
      (node$0.parent = parent),
      node$0),
    init = deserializeOptionExpression(pos + 48);
  if (init !== null) {
    let left = value;
    value =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "AssignmentPattern"),
      (node$0.decorators = []),
      (node$0.left = left),
      (node$0.right = init),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start),
      (node$0.end = end),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0);
    left.parent = value;
    init.parent = value;
  }
  node.key = key;
  node.value = value;
  parent = previousParent;
  return node;
}

function deserializeAssignmentTargetPropertyProperty(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Property"),
      (node$0.kind = "init"),
      (node$0.key = null),
      (node$0.value = null),
      (node$0.method = false),
      (node$0.shorthand = false),
      (node$0.computed = uint8[pos + 12] === 1),
      (node$0.optional = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeAssignmentTargetMaybeDefault(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSequenceExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "SequenceExpression"),
      (node$0.expressions = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expressions = deserializeVecExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeSuper(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Super"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeAwaitExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "AwaitExpression"),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeChainExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ChainExpression"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  node = deserializeExpression(pos + 16);
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExpressionStatement"),
      (node$0.expression = null),
      (node$0.directive = deserializeStr(pos + 64)),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeStringLiteral(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBlockStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "BlockStatement"),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "VariableDeclaration"),
      (node$0.kind = VariableDeclarationKind_TABLE[uint8[pos + 12]]),
      (node$0.declarations = null),
      (node$0.declare = uint8[pos + 13] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.declarations = deserializeVecVariableDeclarator(pos + 16);
  parent = previousParent;
  return node;
}

const VariableDeclarationKind_TABLE = ["var", "let", "const", "using", "await using"];

function deserializeVariableDeclarator(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "VariableDeclarator"),
      (node$0.id = null),
      (node$0.init = null),
      (node$0.definite = uint8[pos + 12] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    pattern = deserializeBindingPattern(pos + 16);
  {
    let previousParent = parent;
    parent = pattern;
    let typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 32);
    if (typeAnnotation !== null) {
      pattern.typeAnnotation = typeAnnotation;
      pattern.range[1] = pattern.end = typeAnnotation.end;
    }
    parent = previousParent;
  }
  node.id = pattern;
  node.init = deserializeOptionExpression(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeEmptyStatement(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "EmptyStatement"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeExpressionStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExpressionStatement"),
      (node$0.expression = null),
      (node$0.directive = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeIfStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "IfStatement"),
      (node$0.test = null),
      (node$0.consequent = null),
      (node$0.alternate = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.test = deserializeExpression(pos + 16);
  node.consequent = deserializeStatement(pos + 32);
  node.alternate = deserializeOptionStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeDoWhileStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "DoWhileStatement"),
      (node$0.body = null),
      (node$0.test = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.body = deserializeStatement(pos + 16);
  node.test = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeWhileStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "WhileStatement"),
      (node$0.test = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.test = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeForStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ForStatement"),
      (node$0.init = null),
      (node$0.test = null),
      (node$0.update = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ForInStatement"),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ForOfStatement"),
      (node$0.await = uint8[pos + 64] === 1),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializeForStatementLeft(pos + 16);
  node.right = deserializeExpression(pos + 32);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeContinueStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ContinueStatement"),
      (node$0.label = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeBreakStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "BreakStatement"),
      (node$0.label = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.label = deserializeOptionLabelIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeReturnStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ReturnStatement"),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeOptionExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeWithStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "WithStatement"),
      (node$0.object = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.object = deserializeExpression(pos + 16);
  node.body = deserializeStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSwitchStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "SwitchStatement"),
      (node$0.discriminant = null),
      (node$0.cases = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.discriminant = deserializeExpression(pos + 16);
  node.cases = deserializeVecSwitchCase(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeSwitchCase(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "SwitchCase"),
      (node$0.test = null),
      (node$0.consequent = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.test = deserializeOptionExpression(pos + 16);
  node.consequent = deserializeVecStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeLabeledStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "LabeledStatement"),
      (node$0.label = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.label = deserializeLabelIdentifier(pos + 16);
  node.body = deserializeStatement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeThrowStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ThrowStatement"),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTryStatement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TryStatement"),
      (node$0.block = null),
      (node$0.handler = null),
      (node$0.finalizer = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.block = deserializeBlockStatement(int32[(pos >> 2) + 4]);
  node.handler = deserializeOptionBoxCatchClause(pos + 24);
  node.finalizer = deserializeOptionBoxBlockStatement(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeCatchClause(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "CatchClause"),
      (node$0.param = null),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
    if (typeAnnotation !== null) {
      pattern.end = typeAnnotation.end;
      pattern.range[1] = typeAnnotation.end;
    }
    parent = previousParent;
  }
  return pattern;
}

function deserializeDebuggerStatement(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "DebuggerStatement"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "AssignmentPattern"),
      (node$0.decorators = []),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializeBindingPattern(pos + 16);
  node.right = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeObjectPattern(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ObjectPattern"),
      (node$0.decorators = []),
      (node$0.properties = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    properties = deserializeVecBindingProperty(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && properties.push(rest);
  node.properties = properties;
  parent = previousParent;
  return node;
}

function deserializeBindingProperty(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Property"),
      (node$0.kind = "init"),
      (node$0.key = null),
      (node$0.value = null),
      (node$0.method = false),
      (node$0.shorthand = uint8[pos + 12] === 1),
      (node$0.computed = uint8[pos + 13] === 1),
      (node$0.optional = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.key = deserializePropertyKey(pos + 16);
  node.value = deserializeBindingPattern(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeArrayPattern(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ArrayPattern"),
      (node$0.decorators = []),
      (node$0.elements = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    elements = deserializeVecOptionBindingPattern(pos + 16),
    rest = deserializeOptionBoxBindingRestElement(pos + 40);
  rest !== null && elements.push(rest);
  node.elements = elements;
  parent = previousParent;
  return node;
}

function deserializeBindingRestElement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "RestElement"),
      (node$0.decorators = []),
      (node$0.argument = null),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.value = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeBindingPattern(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeFunction(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = FunctionType_TABLE[uint8[pos + 88]]),
      (node$0.id = null),
      (node$0.generator = uint8[pos + 89] === 1),
      (node$0.async = uint8[pos + 90] === 1),
      (node$0.declare = uint8[pos + 91] === 1),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.body = null),
      (node$0.expression = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    params = deserializeVecFormalParameter(pos + 16),
    restFieldPos32 = (pos >> 2) + 10;
  if (int32[restFieldPos32] !== 0 && int32[restFieldPos32 + 1] !== 0) {
    pos = int32[restFieldPos32];
    let start,
      end,
      previousParent = parent,
      rest = (parent =
        ((node$0 = Object.create(NodeProto)),
        (node$0.type = "RestElement"),
        (node$0.decorators = []),
        (node$0.argument = null),
        (node$0.optional = false),
        (node$0.typeAnnotation = null),
        (node$0.value = null),
        (node$0.start = start = deserializeI32(pos + 40)),
        (node$0.end = end = deserializeI32(pos + 44)),
        (node$0.range = [start, end]),
        (node$0.parent = previousParent),
        node$0));
    rest.argument = deserializeBindingPattern(pos + 56);
    rest.decorators = deserializeVecDecorator(pos + 16);
    rest.decorators.length !== 0 && (start = rest.decorators[0].start);
    rest.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 72);
    rest.typeAnnotation !== null && (end = rest.typeAnnotation.end);
    rest.start = start;
    rest.end = end;
    rest.range = [start, end];
    params.push(rest);
    parent = previousParent;
  }
  return params;
}

function deserializeFormalParameter(pos) {
  let node$0,
    param,
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
        let start, end;
        param = parent =
          ((node$0 = Object.create(NodeProto)),
          (node$0.type = "AssignmentPattern"),
          (node$0.decorators = null),
          (node$0.left = null),
          (node$0.right = null),
          (node$0.optional = optional),
          (node$0.typeAnnotation = null),
          (node$0.start = start = deserializeI32(pos)),
          (node$0.end = end = deserializeI32(pos + 4)),
          (node$0.range = [start, end]),
          (node$0.parent = previousParent),
          node$0);
        param.decorators = deserializeVecDecorator(pos + 16);
        param.left = deserializeBindingPattern(pos + 40);
        param.left.decorators = [];
        param.left.optional = false;
        parent = param.left;
        let leftTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        param.left.typeAnnotation = leftTypeAnnotation;
        if (leftTypeAnnotation !== null) {
          param.left.end = leftTypeAnnotation.end;
          param.left.range[1] = leftTypeAnnotation.end;
        }
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
        if (typeAnnotation !== null) {
          param.end = typeAnnotation.end;
          param.range[1] = typeAnnotation.end;
        } else if (optional) {
          param.end = deserializeI32(pos + 4);
          param.range[1] = deserializeI32(pos + 4);
        }
        parent = previousParent;
      }
    } else {
      let start, end;
      param = parent =
        ((node$0 = Object.create(NodeProto)),
        (node$0.type = "TSParameterProperty"),
        (node$0.accessibility = accessibility),
        (node$0.decorators = null),
        (node$0.override = override),
        (node$0.parameter = null),
        (node$0.readonly = readonly),
        (node$0.static = false),
        (node$0.start = start = deserializeI32(pos)),
        (node$0.end = end = deserializeI32(pos + 4)),
        (node$0.range = [start, end]),
        (node$0.parent = previousParent),
        node$0);
      param.decorators = deserializeVecDecorator(pos + 16);
      if (hasInitializer) {
        let pattern = deserializeBindingPattern(pos + 40),
          initializer = deserializeOptionBoxExpression(pos + 64),
          assignStart,
          assignEnd,
          assignParam = (parent =
            ((node$0 = Object.create(NodeProto)),
            (node$0.type = "AssignmentPattern"),
            (node$0.decorators = []),
            (node$0.left = null),
            (node$0.right = null),
            (node$0.optional = false),
            (node$0.typeAnnotation = null),
            (node$0.start = assignStart = pattern.start),
            (node$0.end = assignEnd = initializer.end),
            (node$0.range = [assignStart, assignEnd]),
            (node$0.parent = param),
            node$0));
        assignParam.left = pattern;
        pattern.parent = assignParam;
        pattern.decorators = [];
        pattern.optional = false;
        parent = pattern;
        let patternTypeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
        pattern.typeAnnotation = patternTypeAnnotation;
        if (patternTypeAnnotation !== null) {
          pattern.end = patternTypeAnnotation.end;
          pattern.range[1] = patternTypeAnnotation.end;
        }
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
        if (paramTypeAnnotation !== null) {
          param.parameter.end = paramTypeAnnotation.end;
          param.parameter.range[1] = paramTypeAnnotation.end;
        } else if (paramOptional) {
          let paramEnd = deserializeI32(pos + 4);
          param.parameter.end = paramEnd;
          param.parameter.range[1] = paramEnd;
        }
        parent = param;
      }
    }
  }
  parent = previousParent;
  return param;
}

function deserializeFunctionBody(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "BlockStatement"),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ArrowFunctionExpression"),
      (node$0.expression = uint8[pos + 40] !== 64),
      (node$0.async = uint8[pos + 56] === 1),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.body = null),
      (node$0.id = null),
      (node$0.generator = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos >> 2) + 6]);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  node.body = deserializeArrowFunctionBody(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeYieldExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "YieldExpression"),
      (node$0.delegate = uint8[pos + 12] === 1),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.argument = deserializeOptionExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeClass(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = uint8[pos + 136] === 0 ? "ClassDeclaration" : "ClassExpression"),
      (node$0.decorators = null),
      (node$0.id = null),
      (node$0.typeParameters = null),
      (node$0.superClass = null),
      (node$0.superTypeArguments = null),
      (node$0.implements = null),
      (node$0.body = null),
      (node$0.abstract = uint8[pos + 137] === 1),
      (node$0.declare = uint8[pos + 138] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ClassBody"),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = uint8[pos + 12] === 0 ? "MethodDefinition" : "TSAbstractMethodDefinition"),
      (node$0.decorators = null),
      (node$0.key = null),
      (node$0.value = null),
      (node$0.kind = MethodDefinitionKind_TABLE[uint8[pos + 13]]),
      (node$0.computed = uint8[pos + 14] === 1),
      (node$0.static = uint8[pos + 15] === 1),
      (node$0.override = uint8[pos + 64] === 1),
      (node$0.optional = uint8[pos + 65] === 1),
      (node$0.accessibility = deserializeOptionTSAccessibility(pos + 66)),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.value = deserializeFunction(int32[(pos >> 2) + 14]);
  parent = previousParent;
  return node;
}

function deserializePropertyDefinition(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = uint8[pos + 12] === 0 ? "PropertyDefinition" : "TSAbstractPropertyDefinition"),
      (node$0.decorators = null),
      (node$0.key = null),
      (node$0.typeAnnotation = null),
      (node$0.value = null),
      (node$0.computed = uint8[pos + 13] === 1),
      (node$0.static = uint8[pos + 14] === 1),
      (node$0.declare = uint8[pos + 15] === 1),
      (node$0.override = uint8[pos + 80] === 1),
      (node$0.optional = uint8[pos + 81] === 1),
      (node$0.definite = uint8[pos + 82] === 1),
      (node$0.readonly = uint8[pos + 83] === 1),
      (node$0.accessibility = deserializeOptionTSAccessibility(pos + 84)),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  node.value = deserializeOptionExpression(pos + 64);
  parent = previousParent;
  return node;
}

const MethodDefinitionKind_TABLE = ["constructor", "method", "get", "set"];

function deserializePrivateIdentifier(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "PrivateIdentifier"),
    (node$0.name =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start + 1, end)
        : deserializeStr(pos + 16)),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeStaticBlock(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "StaticBlock"),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.body = deserializeVecStatement(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeAccessorProperty(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = uint8[pos + 12] === 0 ? "AccessorProperty" : "TSAbstractAccessorProperty"),
      (node$0.decorators = null),
      (node$0.key = null),
      (node$0.typeAnnotation = null),
      (node$0.value = null),
      (node$0.computed = uint8[pos + 13] === 1),
      (node$0.static = uint8[pos + 14] === 1),
      (node$0.override = uint8[pos + 15] === 1),
      (node$0.definite = uint8[pos + 80] === 1),
      (node$0.accessibility = deserializeOptionTSAccessibility(pos + 81)),
      (node$0.declare = false),
      (node$0.optional = false),
      (node$0.readonly = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.decorators = deserializeVecDecorator(pos + 16);
  node.key = deserializePropertyKey(pos + 40);
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 56);
  node.value = deserializeOptionExpression(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeImportExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ImportExpression"),
      (node$0.source = null),
      (node$0.options = null),
      (node$0.phase = deserializeOptionImportPhase(pos + 12)),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.source = deserializeExpression(pos + 16);
  node.options = deserializeOptionExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeImportDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ImportDeclaration"),
      (node$0.specifiers = null),
      (node$0.source = null),
      (node$0.phase = deserializeOptionImportPhase(pos + 12)),
      (node$0.attributes = null),
      (node$0.importKind = uint8[pos + 13] === 0 ? "value" : "type"),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ImportSpecifier"),
      (node$0.imported = null),
      (node$0.local = null),
      (node$0.importKind = uint8[pos + 12] === 0 ? "value" : "type"),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.imported = deserializeModuleExportName(pos + 16);
  node.local = deserializeBindingIdentifier(pos + 72);
  parent = previousParent;
  return node;
}

function deserializeImportDefaultSpecifier(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ImportDefaultSpecifier"),
      (node$0.local = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.local = deserializeBindingIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeImportNamespaceSpecifier(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ImportNamespaceSpecifier"),
      (node$0.local = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.local = deserializeBindingIdentifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeWithClause(pos) {
  return { attributes: deserializeVecImportAttribute(pos + 16) };
}

function deserializeImportAttribute(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ImportAttribute"),
      (node$0.key = null),
      (node$0.value = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExportNamedDeclaration"),
      (node$0.declaration = null),
      (node$0.specifiers = []),
      (node$0.source = null),
      (node$0.exportKind = null),
      (node$0.attributes = []),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExportNamedDeclaration"),
      (node$0.declaration = null),
      (node$0.specifiers = null),
      (node$0.source = null),
      (node$0.exportKind = uint8[pos + 12] === 0 ? "value" : "type"),
      (node$0.attributes = []),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.specifiers = deserializeVecExportSpecifier(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeExportFromDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExportNamedDeclaration"),
      (node$0.declaration = null),
      (node$0.specifiers = null),
      (node$0.source = null),
      (node$0.exportKind = uint8[pos + 12] === 0 ? "value" : "type"),
      (node$0.attributes = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    withClause = deserializeOptionBoxWithClause(pos + 88);
  node.specifiers = deserializeVecExportSpecifier(pos + 16);
  node.source = deserializeStringLiteral(pos + 40);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportDefaultDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExportDefaultDeclaration"),
      (node$0.declaration = null),
      (node$0.exportKind = "value"),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.declaration = deserializeExportDefaultDeclarationKind(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeExportAllDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExportAllDeclaration"),
      (node$0.exported = null),
      (node$0.source = null),
      (node$0.attributes = null),
      (node$0.exportKind = uint8[pos + 12] === 0 ? "value" : "type"),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    withClause = deserializeOptionBoxWithClause(pos + 120);
  node.exported = deserializeOptionModuleExportName(pos + 16);
  node.source = deserializeStringLiteral(pos + 72);
  node.attributes = withClause === null ? [] : withClause.attributes;
  parent = previousParent;
  return node;
}

function deserializeExportSpecifier(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "ExportSpecifier"),
      (node$0.local = null),
      (node$0.exported = null),
      (node$0.exportKind = uint8[pos + 12] === 0 ? "value" : "type"),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "V8IntrinsicExpression"),
      (node$0.name = null),
      (node$0.arguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.name = deserializeIdentifierName(pos + 16);
  node.arguments = deserializeVecArgument(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeBooleanLiteral(pos) {
  let node$0,
    value = uint8[pos + 12] === 1,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Literal"),
    (node$0.value = value),
    (node$0.raw = start === 0 && end === 0 ? null : value ? "true" : "false"),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeNullLiteral(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Literal"),
    (node$0.value = null),
    (node$0.raw = start === 0 && end === 0 ? null : "null"),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeNumericLiteral(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Literal"),
    (node$0.value = float64[(pos >> 3) + 4]),
    (node$0.raw =
      int32[(pos >> 2) + 4] === 0 && int32[(pos >> 2) + 5] === 0
        ? null
        : sourceText.slice(start, end)),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeStringLiteral(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Literal"),
    (node$0.value =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start + 1, end - 1)
        : deserializeJSStr(pos + 16)),
    (node$0.raw =
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end)),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeBigIntLiteral(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    bigint = deserializeStr(pos + 16);
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "Literal"),
    (node$0.value = BigInt(bigint)),
    (node$0.raw =
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end)),
    (node$0.bigint = bigint),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeRegExpLiteral(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1],
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Literal"),
      (node$0.value = null),
      (node$0.raw =
        int32[(pos >> 2) + 12] === 0 && int32[(pos >> 2) + 13] === 0
          ? null
          : sourceText.slice(start, end)),
      (node$0.regex = null),
      (node$0.start = start),
      (node$0.end = end),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXElement"),
      (node$0.openingElement = null),
      (node$0.children = null),
      (node$0.closingElement = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXOpeningElement"),
      (node$0.name = null),
      (node$0.typeArguments = null),
      (node$0.attributes = null),
      (node$0.selfClosing = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.name = deserializeJSXElementName(pos + 16);
  node.typeArguments = deserializeOptionBoxTSTypeParameterInstantiation(pos + 32);
  node.attributes = deserializeVecJSXAttributeItem(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeJSXClosingElement(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXClosingElement"),
      (node$0.name = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.name = deserializeJSXElementName(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSXFragment(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXFragment"),
      (node$0.openingFragment = null),
      (node$0.children = null),
      (node$0.closingFragment = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.openingFragment = deserializeJSXOpeningFragment(pos + 16);
  node.children = deserializeVecJSXChild(pos + 32);
  node.closingFragment = deserializeJSXClosingFragment(pos + 56);
  parent = previousParent;
  return node;
}

function deserializeJSXOpeningFragment(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "JSXOpeningFragment"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeJSXClosingFragment(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "JSXClosingFragment"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeJSXElementName(pos) {
  let node$0;
  switch (uint8[pos]) {
    case 0:
      return deserializeJSXIdentifier(int32[(pos >> 2) + 2]);
    case 1:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return (
        (node$0 = Object.create(NodeProto)),
        (node$0.type = "JSXIdentifier"),
        (node$0.name = ident.name),
        (node$0.start = ident.start),
        (node$0.end = ident.end),
        (node$0.range = ident.range),
        (node$0.parent = parent),
        node$0
      );
    case 2:
      return deserializeJSXNamespacedName(int32[(pos >> 2) + 2]);
    case 3:
      return deserializeJSXMemberExpression(int32[(pos >> 2) + 2]);
    case 4:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return (
        (node$0 = Object.create(NodeProto)),
        (node$0.type = "JSXIdentifier"),
        (node$0.name = "this"),
        (node$0.start = thisExpr.start),
        (node$0.end = thisExpr.end),
        (node$0.range = thisExpr.range),
        (node$0.parent = parent),
        node$0
      );
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXElementName`);
  }
}

function deserializeJSXNamespacedName(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXNamespacedName"),
      (node$0.namespace = null),
      (node$0.name = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.namespace = deserializeJSXIdentifier(pos + 16);
  node.name = deserializeJSXIdentifier(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeJSXMemberExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXMemberExpression"),
      (node$0.object = null),
      (node$0.property = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.object = deserializeJSXMemberExpressionObject(pos + 16);
  node.property = deserializeJSXIdentifier(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeJSXMemberExpressionObject(pos) {
  let node$0;
  switch (uint8[pos]) {
    case 0:
      let ident = deserializeBoxIdentifierReference(pos + 8);
      return (
        (node$0 = Object.create(NodeProto)),
        (node$0.type = "JSXIdentifier"),
        (node$0.name = ident.name),
        (node$0.start = ident.start),
        (node$0.end = ident.end),
        (node$0.range = ident.range),
        (node$0.parent = parent),
        node$0
      );
    case 1:
      return deserializeJSXMemberExpression(int32[(pos >> 2) + 2]);
    case 2:
      let thisExpr = deserializeBoxThisExpression(pos + 8);
      return (
        (node$0 = Object.create(NodeProto)),
        (node$0.type = "JSXIdentifier"),
        (node$0.name = "this"),
        (node$0.start = thisExpr.start),
        (node$0.end = thisExpr.end),
        (node$0.range = thisExpr.range),
        (node$0.parent = parent),
        node$0
      );
    default:
      throw Error(`Unexpected discriminant ${uint8[pos]} for JSXMemberExpressionObject`);
  }
}

function deserializeJSXExpressionContainer(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXExpressionContainer"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "JSXEmptyExpression"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXAttribute"),
      (node$0.name = null),
      (node$0.value = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.name = deserializeJSXAttributeName(pos + 16);
  node.value = deserializeOptionJSXAttributeValue(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeJSXSpreadAttribute(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXSpreadAttribute"),
      (node$0.argument = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "JSXIdentifier"),
    (node$0.name =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16)),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "JSXSpreadChild"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSXText(pos) {
  let node$0,
    start = int32[pos >> 2],
    end = int32[(pos >> 2) + 1];
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "JSXText"),
    (node$0.value =
      int32[(pos >> 2) + 4] >= sourceStartPos
        ? sourceText.slice(start, end)
        : deserializeStr(pos + 16)),
    (node$0.raw =
      int32[(pos >> 2) + 8] === 0 && int32[(pos >> 2) + 9] === 0
        ? null
        : sourceText.slice(start, end)),
    (node$0.start = start),
    (node$0.end = end),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSThisParameter(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = "this"),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeOptionBoxTSTypeAnnotation(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSEnumDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSEnumDeclaration"),
      (node$0.id = null),
      (node$0.body = null),
      (node$0.const = uint8[pos + 12] === 1),
      (node$0.declare = uint8[pos + 13] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.id = deserializeBindingIdentifier(pos + 16);
  node.body = deserializeTSEnumBody(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSEnumBody(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSEnumBody"),
      (node$0.members = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.members = deserializeVecTSEnumMember(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSEnumMember(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSEnumMember"),
      (node$0.id = null),
      (node$0.initializer = null),
      (node$0.computed = deserializeU8(pos + 16) > 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeAnnotation"),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSLiteralType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSLiteralType"),
      (node$0.literal = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSConditionalType"),
      (node$0.checkType = null),
      (node$0.extendsType = null),
      (node$0.trueType = null),
      (node$0.falseType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.checkType = deserializeTSType(pos + 16);
  node.extendsType = deserializeTSType(pos + 32);
  node.trueType = deserializeTSType(pos + 48);
  node.falseType = deserializeTSType(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeTSUnionType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSUnionType"),
      (node$0.types = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.types = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSIntersectionType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSIntersectionType"),
      (node$0.types = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.types = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSParenthesizedType(pos) {
  let node;
  node = deserializeTSType(pos + 16);
  return node;
}

function deserializeTSTypeOperator(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeOperator"),
      (node$0.operator = TSTypeOperatorOperator_TABLE[uint8[pos + 12]]),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

const TSTypeOperatorOperator_TABLE = ["keyof", "unique", "readonly"];

function deserializeTSArrayType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSArrayType"),
      (node$0.elementType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.elementType = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSIndexedAccessType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSIndexedAccessType"),
      (node$0.objectType = null),
      (node$0.indexType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.objectType = deserializeTSType(pos + 16);
  node.indexType = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTupleType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTupleType"),
      (node$0.elementTypes = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.elementTypes = deserializeVecTSTupleElement(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNamedTupleMember(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSNamedTupleMember"),
      (node$0.label = null),
      (node$0.elementType = null),
      (node$0.optional = uint8[pos + 12] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.label = deserializeIdentifierName(pos + 16);
  node.elementType = deserializeTSTupleElement(pos + 48);
  parent = previousParent;
  return node;
}

function deserializeTSOptionalType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSOptionalType"),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSRestType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSRestType"),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSAnyKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSStringKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSStringKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSBooleanKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSBooleanKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSNumberKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSNumberKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSNeverKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSNeverKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSIntrinsicKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSIntrinsicKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSUnknownKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSUnknownKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSNullKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSNullKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSUndefinedKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSUndefinedKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSVoidKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSVoidKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSSymbolKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSSymbolKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSThisType(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSThisType"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSObjectKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSObjectKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSBigIntKeyword(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSBigIntKeyword"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
}

function deserializeTSTypeReference(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeReference"),
      (node$0.typeName = null),
      (node$0.typeArguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSQualifiedName"),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializeTSTypeName(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterInstantiation(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeParameterInstantiation"),
      (node$0.params = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.params = deserializeVecTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameter(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeParameter"),
      (node$0.name = null),
      (node$0.constraint = null),
      (node$0.default = null),
      (node$0.in = uint8[pos + 12] === 1),
      (node$0.out = uint8[pos + 13] === 1),
      (node$0.const = uint8[pos + 14] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.name = deserializeBindingIdentifier(pos + 16);
  node.constraint = deserializeOptionTSType(pos + 48);
  node.default = deserializeOptionTSType(pos + 64);
  parent = previousParent;
  return node;
}

function deserializeTSTypeParameterDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeParameterDeclaration"),
      (node$0.params = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.params = deserializeVecTSTypeParameter(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAliasDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeAliasDeclaration"),
      (node$0.id = null),
      (node$0.typeParameters = null),
      (node$0.typeAnnotation = null),
      (node$0.declare = uint8[pos + 72] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSClassImplements"),
      (node$0.expression = null),
      (node$0.typeArguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    expression = deserializeTSTypeName(pos + 16);
  if (expression.type === "TSQualifiedName") {
    let object = expression.left,
      { right } = expression,
      start,
      end,
      previous = (expression =
        ((node$0 = Object.create(NodeProto)),
        (node$0.type = "MemberExpression"),
        (node$0.object = object),
        (node$0.property = right),
        (node$0.optional = false),
        (node$0.computed = false),
        (node$0.start = start = expression.start),
        (node$0.end = end = expression.end),
        (node$0.range = [start, end]),
        (node$0.parent = parent),
        node$0));
    right.parent = previous;
    for (;;) {
      if (object.type !== "TSQualifiedName") {
        object.parent = previous;
        break;
      }
      let { left, right } = object;
      previous = previous.object =
        ((node$0 = Object.create(NodeProto)),
        (node$0.type = "MemberExpression"),
        (node$0.object = left),
        (node$0.property = right),
        (node$0.optional = false),
        (node$0.computed = false),
        (node$0.start = start = object.start),
        (node$0.end = end = object.end),
        (node$0.range = [start, end]),
        (node$0.parent = previous),
        node$0);
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSInterfaceDeclaration"),
      (node$0.id = null),
      (node$0.typeParameters = null),
      (node$0.extends = null),
      (node$0.body = null),
      (node$0.declare = uint8[pos + 88] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.id = deserializeBindingIdentifier(pos + 16);
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 48);
  node.extends = deserializeVecTSInterfaceHeritage(pos + 56);
  node.body = deserializeTSInterfaceBody(int32[(pos >> 2) + 20]);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceBody(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSInterfaceBody"),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.body = deserializeVecTSSignature(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSPropertySignature(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSPropertySignature"),
      (node$0.computed = uint8[pos + 12] === 1),
      (node$0.optional = uint8[pos + 13] === 1),
      (node$0.readonly = uint8[pos + 14] === 1),
      (node$0.key = null),
      (node$0.typeAnnotation = null),
      (node$0.accessibility = null),
      (node$0.static = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSIndexSignature"),
      (node$0.parameters = null),
      (node$0.typeAnnotation = null),
      (node$0.readonly = uint8[pos + 12] === 1),
      (node$0.static = uint8[pos + 13] === 1),
      (node$0.accessibility = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.parameters = [deserializeTSIndexSignatureName(pos + 16)];
  node.typeAnnotation = deserializeTSTypeAnnotation(int32[(pos >> 2) + 14]);
  parent = previousParent;
  return node;
}

function deserializeTSCallSignatureDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSCallSignatureDeclaration"),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSMethodSignature"),
      (node$0.key = null),
      (node$0.computed = uint8[pos + 64] === 1),
      (node$0.optional = uint8[pos + 65] === 1),
      (node$0.kind = TSMethodSignatureKind_TABLE[uint8[pos + 66]]),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.accessibility = null),
      (node$0.readonly = false),
      (node$0.static = false),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSConstructSignatureDeclaration"),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos >> 2) + 6]);
  node.returnType = deserializeOptionBoxTSTypeAnnotation(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSIndexSignatureName(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Identifier"),
      (node$0.decorators = []),
      (node$0.name = deserializeStr(pos + 16)),
      (node$0.optional = false),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSTypeAnnotation(int32[(pos >> 2) + 8]);
  parent = previousParent;
  return node;
}

function deserializeTSInterfaceHeritage(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSInterfaceHeritage"),
      (node$0.expression = null),
      (node$0.typeArguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    expression = deserializeTSTypeName(pos + 16);
  if (expression.type === "TSQualifiedName") {
    let object = expression.left,
      { right } = expression,
      start,
      end,
      previous = (expression =
        ((node$0 = Object.create(NodeProto)),
        (node$0.type = "MemberExpression"),
        (node$0.object = object),
        (node$0.property = right),
        (node$0.optional = false),
        (node$0.computed = false),
        (node$0.start = start = expression.start),
        (node$0.end = end = expression.end),
        (node$0.range = [start, end]),
        (node$0.parent = parent),
        node$0));
    right.parent = previous;
    for (;;) {
      if (object.type !== "TSQualifiedName") {
        object.parent = previous;
        break;
      }
      let { left, right } = object;
      previous = previous.object =
        ((node$0 = Object.create(NodeProto)),
        (node$0.type = "MemberExpression"),
        (node$0.object = left),
        (node$0.property = right),
        (node$0.optional = false),
        (node$0.computed = false),
        (node$0.start = start = object.start),
        (node$0.end = end = object.end),
        (node$0.range = [start, end]),
        (node$0.parent = previous),
        node$0);
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypePredicate"),
      (node$0.parameterName = null),
      (node$0.asserts = uint8[pos + 12] === 1),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    declare = deserializeBool(pos + 72),
    previousParent = parent,
    body = deserializeOptionBoxTSModuleBlock(pos + 64),
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSModuleDeclaration"),
      (node$0.id = null),
      body !== null && (node$0.body = body),
      (node$0.kind = "module"),
      (node$0.declare = declare),
      (node$0.global = false),
      (node$0.start = start),
      (node$0.end = end),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.id = deserializeStringLiteral(pos + 16);
  body !== null && (body.parent = node);
  parent = previousParent;
  return node;
}

function deserializeTSNamespaceDeclaration(pos) {
  let node$0,
    kind = deserializeTSNamespaceDeclarationKind(pos + 64),
    start = deserializeI32(pos),
    end = deserializeI32(pos + 4),
    declare = deserializeBool(pos + 65),
    previousParent = parent,
    body = deserializeTSNamespaceDeclarationBody(pos + 48),
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSModuleDeclaration"),
      (node$0.id = null),
      (node$0.body = body),
      (node$0.kind = kind),
      (node$0.declare = declare),
      (node$0.global = false),
      (node$0.start = start),
      (node$0.end = end),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    id = deserializeBindingIdentifier(pos + 16);
  if (body.type === "TSModuleBlock") {
    node.id = id;
    body.parent = node;
  } else {
    let innerId = body.id;
    if (innerId.type === "Identifier") {
      let start,
        end,
        outerId =
          (node.id =
          parent =
            ((node$0 = Object.create(NodeProto)),
            (node$0.type = "TSQualifiedName"),
            (node$0.left = id),
            (node$0.right = innerId),
            (node$0.start = start = id.start),
            (node$0.end = end = innerId.end),
            (node$0.range = [start, end]),
            (node$0.parent = node),
            node$0));
      id.parent = innerId.parent = outerId;
    } else {
      // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with
      // `id` of this namespace on left, and previous `left` on right.
      node.id = innerId;
      innerId.parent = node;
      let { start } = id;
      for (;;) {
        innerId.start = innerId.range[0] = start;
        if (innerId.left.type === "Identifier") break;
        innerId = innerId.left;
      }
      let end,
        right = innerId.left;
      id.parent =
        right.parent =
        innerId.left =
          ((node$0 = Object.create(NodeProto)),
          (node$0.type = "TSQualifiedName"),
          (node$0.left = id),
          (node$0.right = right),
          (node$0.start = start),
          (node$0.end = end = right.end),
          (node$0.range = [start, end]),
          (node$0.parent = innerId),
          node$0);
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSModuleDeclaration"),
      (node$0.id = null),
      (node$0.body = null),
      (node$0.kind = "global"),
      (node$0.declare = uint8[pos + 88] === 1),
      (node$0.global = true),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
    keywordStart,
    keywordEnd;
  node.id =
    ((node$0 = Object.create(NodeProto)),
    (node$0.type = "Identifier"),
    (node$0.decorators = []),
    (node$0.name = "global"),
    (node$0.optional = false),
    (node$0.typeAnnotation = null),
    (node$0.start = keywordStart = deserializeI32(pos + 16)),
    (node$0.end = keywordEnd = deserializeI32(pos + 20)),
    (node$0.range = [keywordStart, keywordEnd]),
    (node$0.parent = parent),
    node$0);
  node.body = deserializeTSModuleBlock(pos + 24);
  parent = previousParent;
  return node;
}

function deserializeTSModuleBlock(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSModuleBlock"),
      (node$0.body = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeLiteral"),
      (node$0.members = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.members = deserializeVecTSSignature(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSInferType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSInferType"),
      (node$0.typeParameter = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeParameter = deserializeTSTypeParameter(int32[(pos >> 2) + 4]);
  parent = previousParent;
  return node;
}

function deserializeTSTypeQuery(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeQuery"),
      (node$0.exprName = null),
      (node$0.typeArguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSImportType"),
      (node$0.source = null),
      (node$0.options = null),
      (node$0.qualifier = null),
      (node$0.typeArguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSQualifiedName"),
      (node$0.left = null),
      (node$0.right = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.left = deserializeTSImportTypeQualifier(pos + 16);
  node.right = deserializeIdentifierName(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSFunctionType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSFunctionType"),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSConstructorType"),
      (node$0.abstract = uint8[pos + 40] === 1),
      (node$0.typeParameters = null),
      (node$0.params = null),
      (node$0.returnType = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeParameters = deserializeOptionBoxTSTypeParameterDeclaration(pos + 16);
  node.params = deserializeFormalParameters(int32[(pos >> 2) + 6]);
  node.returnType = deserializeTSTypeAnnotation(int32[(pos >> 2) + 8]);
  parent = previousParent;
  return node;
}

function deserializeTSMappedType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSMappedType"),
      (node$0.key = null),
      (node$0.constraint = null),
      (node$0.nameType = null),
      (node$0.typeAnnotation = null),
      (node$0.optional = null),
      (node$0.readonly = deserializeOptionTSMappedTypeModifierOperator(pos + 97)),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0)),
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTemplateLiteralType"),
      (node$0.quasis = null),
      (node$0.types = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.quasis = deserializeVecTemplateElement(pos + 16);
  node.types = deserializeVecTSType(pos + 40);
  parent = previousParent;
  return node;
}

function deserializeTSAsExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSAsExpression"),
      (node$0.expression = null),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSSatisfiesExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSSatisfiesExpression"),
      (node$0.expression = null),
      (node$0.typeAnnotation = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  node.typeAnnotation = deserializeTSType(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSTypeAssertion(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSTypeAssertion"),
      (node$0.typeAnnotation = null),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSType(pos + 16);
  node.expression = deserializeExpression(pos + 32);
  parent = previousParent;
  return node;
}

function deserializeTSImportEqualsDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSImportEqualsDeclaration"),
      (node$0.id = null),
      (node$0.moduleReference = null),
      (node$0.importKind = uint8[pos + 12] === 0 ? "value" : "type"),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
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
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSExternalModuleReference"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeStringLiteral(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNonNullExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSNonNullExpression"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeDecorator(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "Decorator"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSExportAssignment(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSExportAssignment"),
      (node$0.expression = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSNamespaceExportDeclaration(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSNamespaceExportDeclaration"),
      (node$0.id = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.id = deserializeIdentifierName(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeTSInstantiationExpression(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSInstantiationExpression"),
      (node$0.expression = null),
      (node$0.typeArguments = null),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.expression = deserializeExpression(pos + 16);
  node.typeArguments = deserializeTSTypeParameterInstantiation(int32[(pos >> 2) + 8]);
  parent = previousParent;
  return node;
}

function deserializeJSDocNullableType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSJSDocNullableType"),
      (node$0.typeAnnotation = null),
      (node$0.postfix = uint8[pos + 12] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSDocNonNullableType(pos) {
  let node$0,
    start,
    end,
    previousParent = parent,
    node = (parent =
      ((node$0 = Object.create(NodeProto)),
      (node$0.type = "TSJSDocNonNullableType"),
      (node$0.typeAnnotation = null),
      (node$0.postfix = uint8[pos + 12] === 1),
      (node$0.start = start = int32[pos >> 2]),
      (node$0.end = end = int32[(pos >> 2) + 1]),
      (node$0.range = [start, end]),
      (node$0.parent = parent),
      node$0));
  node.typeAnnotation = deserializeTSType(pos + 16);
  parent = previousParent;
  return node;
}

function deserializeJSDocUnknownType(pos) {
  let node$0, start, end;
  return (
    (node$0 = Object.create(NodeProto)),
    (node$0.type = "TSJSDocUnknownType"),
    (node$0.start = start = int32[pos >> 2]),
    (node$0.end = end = int32[(pos >> 2) + 1]),
    (node$0.range = [start, end]),
    (node$0.parent = parent),
    node$0
  );
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
