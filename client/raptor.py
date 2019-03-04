import json
import requests
import sys


class Store :
    def __init__(self, url):
        '''
            Connect to a Raptor Store.

            A Raptor Store holds multiple documents, and is made available by
            raptor-api.
        '''
        self._url = url

    @property
    def url(self):
        '''
            Get the url to the raptor-api instance holding this Store.
        '''
        return self._url
    

    def request(self, method, params):
        '''
            Perform a raw jsonrpc request to the raptor-api instance.
        '''
        headers = {'content-type': 'application/json'}

        payload = {
            'method': method,
            'params': params,
            'jsonrpc': '2.0',
            'id': 0
        }

        data = json.dumps(payload)

        response = requests.post(self.url, data=data, headers=headers).json()

        if 'error' in response:
            print(response['error'])

        assert response["jsonrpc"]
        assert response["id"] == 0
        return response["result"]

    @property
    def documents(self):
        '''
            Return a list of all documents
        '''
        return [Document(self, d) for d in self.request('documents', {})]

    def document(self, name):
        '''
            Get a specific document by name.
        '''
        return Document(self, name)

    def upload_binary(self, name, filename):
        '''
            Upload a binary to the store.

            This creates and returns a new document.
        '''
        fh = open(filename, 'rb')
        data = fh.read()
        fh.close()
        bytes = [int(x) for x in data]
        return self.new_document(name, bytes)

    def new_document(self, name, bytes):
        '''
            Create a new document.

            Bytes must be an array of integers representing each byte in the
            file you are uploading.
        '''
        self.request('document-new', {'name': name, 'bytes': bytes})
        return Document(self, name)


class Document:
    def __init__(self, store, name):
        self._store = store
        self._name = name

    @property
    def store(self):
        return self._store
    
    @property
    def name(self):
        return self._name
    

    @property
    def functions(self):
        response = self.store.request(
            'document-functions',
            {'document-name': self.name})
        return [Function(self, function['index']) for function in response]

    def xrefs(self):
        return self.store.request(
            'document-xrefs',
            {'document-name': self.name})

    def function_by_name(self, name):
        for function in self.functions:
            if function.name == name:
                return function
        return None

    def function(self, index):
        return Function(
            self,
            self.store.request('function-ir',
                {'document-name': self.name, 'function-index': index}))

    def calls_to_symbol(self, symbol):
        calls = self.store.request(
            'calls-to-symbol',
            {'document-name': self.name, 'symbol': symbol})
        return [ProgramLocation(self, call) for call in calls]


class Function:
    def __init__(self, document, json):
        self._document = document
        self._json = json

    @property
    def document(self):
        return self._document
    
    @property
    def index(self):
        return self._json['index']

    @property
    def address(self):
        return self._json['address']

    @property
    def name(self):
        return self._json['name']

    @property
    def blocks(self):
        return [Block(self, b) for b in self._json['blocks']]

    def block(self, index):
        for block in self.blocks:
            if block.index == index:
                return block
        return None


class Block:
    def __init__(self, function, json):
        self._function = function
        self._json = json

    @property
    def function(self):
        return self._function

    @property
    def document(self):
        return self.function.document

    @property
    def index(self):
        return self._json['index']

    @property
    def json(self):
        return self._json

    @property
    def instructions(self):
        return [Instruction(self, i) for i in self._json['instructions']]

    def instruction(self, index):
        for instruction in self.instructions:
            if instruction.index == index:
                return instruction
        return None


class Instruction:
    def __init__(self, block, json):
        self._block = block
        self._json = json

    @property
    def block(self):
        return self._block

    @property
    def index(self):
        return self._json['index']

    @property
    def address(self):
        return self._json['address']

    @property
    def operation(self):
        return Operation(self._json['operation'])

    @property
    def json(self):
        return self._json

    def __str__(self):
        return '{:x} {:02x} {}'.format(self.address, self.index, self.operation)


class Operation:
    def __init__(self, json):
        self._json = json

    @property
    def operation(self):
        return self._json['operation']

    @property
    def dst(self):
        if 'dst' in self._json:
            return Variable(self._json['dst'])
        else:
            return None

    @property
    def src(self):
        if 'src' in self._json:
            return Expression(self._json['src'])
        else:
            return None

    @property
    def target(self):
        if 'target' in self._json:
            return Expression(self._json['target'])
        else:
            return None

    @property
    def index(self):
        if 'index' in self._json:
            return Expression(self._json['index'])
        else:
            return None

    @property
    def call(self):
        if 'call' in self._json:
            return Call(self._json['call'])
        else:
            return None

    @property
    def intrinsic(self):
        if 'intrinsic' in self._json:
            return Intrinsic(self._json['intrinsic'])
        else:
            return None

    def __str__(self):
        if self.operation == 'assign':
            return '{} = {}'.format(self.dst, self.src)
        elif self.operation == 'load':
            return '{} = [{}]'.format(self.dst, self.index)
        elif self.operation == 'store':
            return '[{}] = {}'.format(self.index, self.src)
        elif self.operation == 'branch':
            return 'branch {}'.format(self.target)
        elif self.operation == 'call':
            return str(self.call)
        elif self.operation == 'intrinsic':
            return str(self.intrinsic)
        elif self.operation == 'return':
            return str(Return(self._json['return']))
        elif self.operation == 'nop':
            return 'nop'


class Return:
    def __init__(self, json):
        self._json = json

    def __str__(self):
        if 'result' in self._json:
            return 'return {}'.format(Expression(self._json['result']))
        else:
            return 'return'

    def __repr__(self):
        return str(self)


class Intrinsic:
    def __init__(self, json):
        self._json = json

    def __str__(self):
        return self._json['instruction_str']

    def __repr__(self):
        return str(self)


class CallTarget:
    def __init__(self, json):
        self._json = json

    @property
    def expression(self):
        return self._json['expression'] if 'expression' in self._json else None

    @property
    def symbol(self):
        return self._json['symbol'] if 'symbol' in self._json else None

    @property
    def function_id(self):
        if 'function_id' in self._json:
            return self._json['function_id']
        else:
            return None


class Call:
    def __init__(self, json):
        self._json = json

    @property
    def call_target(self):
        return CallTarget(self._json['target'])

    @property
    def arguments(self):
        return [Expression(x) for x in self._json['arguments']]

    def __str__(self):
        if self.call_target.expression:
            return 'call {}'.format(self.call_target.expression)
        elif self.call_target.symbol:
            return '{}({})'.format(
                self.call_target.symbol,
                ', '.join([str(x) for x in self.arguments]))
        elif self.call_target.function_id:
            return 'id_{}({})'.format(hex(self.call_target.function_id),
                ', '.join([str(x) for x in self.arguments]))

    def __repr__(self):
        return str(self)


class Expression:
    def __init__(self, json):
        self._json = json

    @property
    def lhs(self):
        if 'lhs' in self._json:
            return Expression(self._json['lhs'])
        else:
            return None

    @property
    def rhs(self):
        if 'rhs' in self._json:
            return Expression(self._json['rhs'])
        else:
            return None

    def __str__(self):
        if 'type' in self._json:
            if self._json['type'] == 'scalar':
                return str(Scalar(self._json))
            elif self._json['type'] == 'stack_variable':
                return str(StackVariable(self._json))
            elif self._json['type'] == 'constant':
                return str(Constant(self._json))
            elif self._json['type'] == 'reference':
                return '&({})'.format(Expression(self._json['expression']))
            elif self._json['type'] == 'dereference':
                return '*({})'.format(Expression(self._json['expression']))
            else:
                raise "Unhandled expression type"
        else:
            ops = {
                'add': lambda l, r: '({} + {})'.format(l, r),
                'sub': lambda l, r: '({} - {})'.format(l, r),
                'mul': lambda l, r: '({} * {})'.format(l, r),
                'divu': lambda l, r: '({} /u {})'.format(l, r),
                'modu': lambda l, r: '({} \%u {})'.format(l, r),
                'divs': lambda l, r: '({} /s {})'.format(l, r),
                'mods': lambda l, r: '({} \%s {})'.format(l, r),
                'and': lambda l, r: '({} & {})'.format(l, r),
                'or': lambda l, r: '({} | {})'.format(l, r),
                'xor': lambda l, r: '({} ^ {})'.format(l, r),
                'shl': lambda l, r: '({} << {})'.format(l, r),
                'shr': lambda l, r: '({} >> {})'.format(l, r),
                'cmpeq': lambda l, r: '({} == {})'.format(l, r),
                'cmpneq': lambda l, r: '({} != {})'.format(l, r),
                'cmplts': lambda l, r: '({} <s {})'.format(l, r),
                'cmpltu': lambda l, r: '({} <u {})'.format(l, r)
            }
            op = self._json['op']
            if op in ops:
                return ops[op](self.lhs, self.rhs)
            elif op == 'trun':
                return 'trun.{}({})'.format(self._json['bits'], self.rhs)
            elif op == 'sext':
                return 'sext.{}({})'.format(self._json['bits'], self.rhs)
            elif op == 'zext':
                return 'zext.{}({})'.format(self._json['bits'], self.rhs)
            elif op == 'ite':
                return 'ite({}, {}, {})'.format(
                    Expression(self._json['cond']),
                    Expression(self._json['then']),
                    Expression(self._json['else']))

    def __repr__(self):
        return str(self)


class Variable:
    def __init__(self, json):
        self._json = json

    def __str__(self):
        if self._json['type'] == 'scalar':
            return str(Scalar(self._json))
        elif self._json['type'] == 'stack_variable':
            return str(StackVariable(self._json))

    def __repr__(self):
        return str(self)


class Scalar:
    def __init__(self, json):
        self._json = json

    @property
    def bits(self):
        return self._json['bits']

    @property
    def name(self):
        return self._json['name']

    def __str__(self):
        return '{}:{}'.format(self.name, self.bits)

    def __repr__(self):
        return str(self)


class StackVariable:
    def __init__(self, json):
        self._json = json

    @property
    def offset(self):
        return self._json['offset']

    @property
    def bits(self):
        return self._json['bits']

    def __str__(self):
        if self.offset < 0:
            return 'var_{}:{}'.format(hex(self.offset * -1), self.bits)
        else:
            return 'arg_{}:{}'.format(hex(self.offset), self.bits)

    def __repr__(self):
        return str(self)


class Constant:
    def __init__(self, json):
        self._json = json

    @property
    def bits(self):
        return self._json['bits']

    @property
    def value(self):
        return int(self._json['value'], 16)

    def __str__(self):
        return '{}:{}'.format(hex(self.value), self.bits)

    def __repr__(self):
        return str(self)


class FunctionLocation:
    def __init__(self, document, json, program_location=None):
        self._document = document
        self._json = json
        self._program_location = program_location

    @property
    def document(self):
        return self._document

    @property
    def json(self):
        return self._json

    @property
    def block_index(self):
        if 'block-index' in self._json:
            return self._json['block-index']
        else:
            return None

    @property
    def instruction_index(self):
        if 'instruction-index' in self._json:
            return self._json['instruction-index']
        else:
            return None

    @property
    def block(self):
        if self._program_location == None or self.block_index == None:
            return None
        self._program_location.function.block(self.block_index)

    @property
    def instruction(self):
        if self._program_location == None or \
           self.block_index == None or \
           self.instruction_index == None:
            return None

        return self._program_location\
            .function\
            .block(self.block_index)\
            .instruction(self.instruction_index)


class ProgramLocation:
    def __init__(self, document, json):
        self._document = document
        self._json = json

    @property
    def document(self):
        return self._document

    @property
    def json(self):
        return self._json

    @property
    def function(self):
        return self.document.function(self._json['function-index'])

    @property
    def function_location(self):
        return FunctionLocation(self.document,
                                self._json['function-location'],
                                self)

    @property
    def block(self):
        return self.function_location.block

    @property
    def instruction(self):
        return self.function_location.instruction


if __name__ == '__main__':
    store = Store('http://localhost:3030/')
    store.upload_binary(sys.argv[1], sys.argv[2])