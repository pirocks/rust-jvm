% incorrect warnings supression
:- discontiguous instructionHasEquivalentTypeRule/2.
:- discontiguous instructionIsTypeSafe/6.
:- discontiguous initHandlerIsLegal/2.

% prolog defs



classIsTypeSafe(Class) :-
    classClassName(Class, Name),
    classDefiningLoader(Class, L),
    superclassChain(Name, L, Chain),
    Chain \= [],
    classSuperClassName(Class, SuperclassName),
    loadedClass(SuperclassName, L, Superclass),
    classIsNotFinal(Superclass),
    classMethods(Class, Methods),
    checklist(methodIsTypeSafe(Class), Methods).

classIsTypeSafe(Class) :-
    classClassName(Class, 'java/lang/Object'),
    classDefiningLoader(Class, L),
    isBootstrapLoader(L),
    classMethods(Class, Methods),
    checklist(methodIsTypeSafe(Class), Methods).

allInstructions(Environment, Instructions) :-
    Environment = environment(_Class, _Method, _ReturnType, Instructions, _, _).

exceptionHandlers(Environment, Handlers) :-
    Environment = environment(_Class, _Method, _ReturnType, _Instructions, _, Handlers).

maxOperandStackLength(Environment, MaxStack) :-
    Environment = environment(_Class, _Method, _ReturnType,_Instructions, MaxStack, _Handlers).

thisClass(Environment, class(ClassName, L)) :-
    Environment = environment(Class, _Method, _ReturnType,_Instructions, _, _),
    classDefiningLoader(Class, L),
    classClassName(Class, ClassName).

thisMethodReturnType(Environment, ReturnType) :-
    Environment = environment(_Class, _Method, ReturnType,_Instructions, _, _).

offsetStackFrame(Environment, Offset, StackFrame) :-
    allInstructions(Environment, Instructions),
    member(stackMap(Offset, StackFrame), Instructions).

currentClassLoader(Environment, Loader) :-
    thisClass(Environment, class(_, Loader)).

notMember(_, []).

notMember(X, [A | More]) :- X \= A, notMember(X, More).
%
%isAssignable(X,X).
%
%isAssignable(oneWord, top).
%isAssignable(twoWord, top).
%
%isAssignable(int, X) :- isAssignable(oneWord, X).
%isAssignable(float, X) :- isAssignable(oneWord, X).
%isAssignable(long, X) :- isAssignable(twoWord, X).
%isAssignable(double, X) :- isAssignable(twoWord, X).
%
%isAssignable(reference, X) :- isAssignable(oneWord, X).
%isAssignable(class(_, _), X) :- isAssignable(reference, X).
%isAssignable(arrayOf(_), X) :- isAssignable(reference, X).
%
%isAssignable(uninitialized, X) :- isAssignable(reference, X).
%isAssignable(uninitializedThis, X) :- isAssignable(uninitialized, X).
%isAssignable(uninitialized(_), X) :- isAssignable(uninitialized, X).
%
%isAssignable(null, class(_, _)).
%isAssignable(null, arrayOf(_)).
%isAssignable(null, X) :-
%    isAssignable(class('java/lang/Object', BL), X),
%    isBootstrapLoader(BL).
%
%
%isAssignable(class(X, Lx), class(Y, Ly)) :-
%    isJavaAssignable(class(X, Lx), class(Y, Ly)).
%
%isAssignable(arrayOf(X), class(Y, L)) :-
%    isJavaAssignable(arrayOf(X), class(Y, L)).
%
%isAssignable(arrayOf(X), arrayOf(Y)) :-
%    isJavaAssignable(arrayOf(X), arrayOf(Y)).

isJavaAssignable(class(_, _), class(To, L)) :-
    loadedClass(To, L, ToClass),
    classIsInterface(ToClass).

isJavaAssignable(From, To) :-
    isJavaSubclassOf(From, To).

isJavaAssignable(arrayOf(_), class('java/lang/Object', BL)) :-
    isBootstrapLoader(BL).

isJavaAssignable(arrayOf(_), X) :-
    isArrayInterface(X).

isJavaAssignable(arrayOf(X), arrayOf(Y)) :-
    atom(X),
    atom(Y),
    X = Y.

isJavaAssignable(arrayOf(X), arrayOf(Y)) :-
    compound(X), compound(Y), isJavaAssignable(X, Y).

isArrayInterface(class('java/lang/Cloneable', BL)) :-
    isBootstrapLoader(BL).

isArrayInterface(class('java/io/Serializable', BL)) :-
    isBootstrapLoader(BL).

isJavaSubclassOf(class(SubclassName, L), class(SubclassName, L)).

isJavaSubclassOf(class(SubclassName, LSub), class(SuperclassName, LSuper)) :-
    superclassChain(SubclassName, LSub, Chain),
    member(class(SuperclassName, L), Chain),
    loadedClass(SuperclassName, L, Sup),
    loadedClass(SuperclassName, LSuper, Sup).

superclassChain(ClassName, L, [class(SuperclassName, Ls) | Rest]) :-
    loadedClass(ClassName, L, Class),
    classSuperClassName(Class, SuperclassName),
    classDefiningLoader(Class, Ls),
    superclassChain(SuperclassName, Ls, Rest).

superclassChain('java/lang/Object', L, []) :-
    loadedClass('java/lang/Object', L, Class),
    classDefiningLoader(Class, BL),
    isBootstrapLoader(BL).


%info that needs to be fed to prolog
instruction(Offset, AnInstruction).
instruction(21, aload(1)).
stackMap(Offset, TypeState).

%prolg defs
frameIsAssignable(frame(Locals1, StackMap1, Flags1), frame(Locals2, StackMap2, Flags2)) :-
    length(StackMap1, StackMapLength),
    length(StackMap2, StackMapLength),
    maplist(isAssignable, Locals1, Locals2),
    maplist(isAssignable, StackMap1, StackMap2),
    subset(Flags1, Flags2).

validTypeTransition(Environment, ExpectedTypesOnStack, ResultType,frame(Locals, InputOperandStack, Flags),frame(Locals, NextOperandStack, Flags)) :-
    popMatchingList(InputOperandStack, ExpectedTypesOnStack,InterimOperandStack),
    pushOperandStack(InterimOperandStack, ResultType, NextOperandStack),
    operandStackHasLegalLength(Environment, NextOperandStack).

popMatchingList(OperandStack, [], OperandStack).

popMatchingList(OperandStack, [P | Rest], NewOperandStack) :-
    popMatchingType(OperandStack, P, TempOperandStack, _ActualType),
    popMatchingList(TempOperandStack, Rest, NewOperandStack).

popMatchingType([ActualType | OperandStack],Type, OperandStack, ActualType) :-
    sizeOf(Type, 1),
    isAssignable(ActualType, Type).

popMatchingType([top, ActualType | OperandStack],Type, OperandStack, ActualType) :-
    sizeOf(Type, 2),
    isAssignable(ActualType, Type).

sizeOf(X, 2) :- isAssignable(X, twoWord).

sizeOf(X, 1) :- isAssignable(X, oneWord).

sizeOf(top, 1).


pushOperandStack(OperandStack, 'void', OperandStack).

pushOperandStack(OperandStack, Type, [Type | OperandStack]) :-
    sizeOf(Type, 1).

pushOperandStack(OperandStack, Type, [top, Type | OperandStack]) :-
    sizeOf(Type, 2).

operandStackHasLegalLength(Environment, OperandStack) :-
    length(OperandStack, Length),
    maxOperandStackLength(Environment, MaxStack),
    Length =< MaxStack.


popCategory1([Type | Rest], Type, Rest) :-
    Type \= top,
    sizeOf(Type, 1).

popCategory2([top, Type | Rest], Type, Rest) :-
    sizeOf(Type, 2).

canSafelyPush(Environment, InputOperandStack, Type, OutputOperandStack) :-
    pushOperandStack(InputOperandStack, Type, OutputOperandStack),
    operandStackHasLegalLength(Environment, OutputOperandStack).

canSafelyPushList(Environment, InputOperandStack, Types,
    OutputOperandStack) :-
    canPushList(InputOperandStack, Types, OutputOperandStack),
    operandStackHasLegalLength(Environment, OutputOperandStack).

canPushList(InputOperandStack, [], InputOperandStack).

canPushList(InputOperandStack, [Type | Rest], OutputOperandStack) :-
    pushOperandStack(InputOperandStack, Type, InterimOperandStack),
    canPushList(InterimOperandStack, Rest, OutputOperandStack).

canPop(frame(Locals, OperandStack, Flags), Types, frame(Locals, PoppedOperandStack, Flags)) :-
    popMatchingList(OperandStack, Types, PoppedOperandStack).


nth1OperandStackIs(I, frame(_Locals, OperandStack, _Flags), Element) :-
    nth1(I, OperandStack, Element).



doesNotOverrideFinalMethod(class('java/lang/Object', L), Method) :-
    isBootstrapLoader(L).

doesNotOverrideFinalMethod(Class, Method) :-
    isPrivate(Method, Class).

doesNotOverrideFinalMethod(Class, Method) :-
    isStatic(Method, Class).

doesNotOverrideFinalMethod(Class, Method) :-
    isNotPrivate(Method, Class),
    isNotStatic(Method, Class),
    doesNotOverrideFinalMethodOfSuperclass(Class, Method).

doesNotOverrideFinalMethodOfSuperclass(Class, Method) :-
    classSuperClassName(Class, SuperclassName),
    classDefiningLoader(Class, L),
    loadedClass(SuperclassName, L, Superclass),
    classMethods(Superclass, SuperMethodList),
    finalMethodNotOverridden(Method, Superclass, SuperMethodList).

finalMethodNotOverridden(Method, Superclass, SuperMethodList) :-
    methodName(Method, Name),
    methodDescriptor(Method, Descriptor),
    member(method(_, Name, Descriptor), SuperMethodList),
    isFinal(Method, Superclass),
    isPrivate(Method, Superclass).

finalMethodNotOverridden(Method, Superclass, SuperMethodList) :-
    methodName(Method, Name),
    methodDescriptor(Method, Descriptor),
    member(method(_, Name, Descriptor), SuperMethodList),
    isFinal(Method, Superclass),
    isStatic(Method, Superclass).

finalMethodNotOverridden(Method, Superclass, SuperMethodList) :-
    methodName(Method, Name),
    methodDescriptor(Method, Descriptor),
    member(method(_, Name, Descriptor), SuperMethodList),
    isNotFinal(Method, Superclass),
    isPrivate(Method, Superclass),
    doesNotOverrideFinalMethod(Superclass, Method).

finalMethodNotOverridden(Method, Superclass, SuperMethodList) :-
    methodName(Method, Name),
    methodDescriptor(Method, Descriptor),
    member(method(_, Name, Descriptor), SuperMethodList),
    isNotFinal(Method, Superclass),
    isStatic(Method, Superclass),
    doesNotOverrideFinalMethod(Superclass, Method).

finalMethodNotOverridden(Method, Superclass, SuperMethodList) :-
    methodName(Method, Name),
    methodDescriptor(Method, Descriptor),
    member(method(_, Name, Descriptor), SuperMethodList),
    isNotFinal(Method, Superclass),
    isNotStatic(Method, Superclass),
    isNotPrivate(Method, Superclass).

finalMethodNotOverridden(Method, Superclass, SuperMethodList) :-
    methodName(Method, Name),
    methodDescriptor(Method, Descriptor),
    notMember(method(_, Name, Descriptor), SuperMethodList),
    doesNotOverrideFinalMethod(Superclass, Method).


methodIsTypeSafe(Class, Method) :-
    doesNotOverrideFinalMethod(Class, Method),
    methodAccessFlags(Method, AccessFlags),
    methodAttributes(Method, Attributes),
    notMember(native, AccessFlags),
    notMember(abstract, AccessFlags),
    member(attribute('Code', _), Attributes),
    methodWithCodeIsTypeSafe(Class, Method).

methodIsTypeSafe(Class, Method) :-
    doesNotOverrideFinalMethod(Class, Method),
    methodAccessFlags(Method, AccessFlags),
    member(abstract, AccessFlags).

methodIsTypeSafe(Class, Method) :-
    doesNotOverrideFinalMethod(Class, Method),
    methodAccessFlags(Method, AccessFlags),
    member(native, AccessFlags).

methodWithCodeIsTypeSafe(Class, Method) :-
    parseCodeAttribute(Class, Method, FrameSize, MaxStack,ParsedCode, Handlers, StackMap),
    mergeStackMapAndCode(StackMap, ParsedCode, MergedCode),
    methodInitialStackFrame(Class, Method, FrameSize, StackFrame, ReturnType),
    Environment = environment(Class, Method, ReturnType, MergedCode,MaxStack, Handlers),
    handlersAreLegal(Environment),
    mergedCodeIsTypeSafe(Environment, MergedCode, StackFrame).

handlersAreLegal(Environment) :-
    exceptionHandlers(Environment, Handlers),
    checklist(handlerIsLegal(Environment), Handlers).
handlerIsLegal(Environment, Handler) :-
    Handler = handler(Start, End, Target, _),
    Start < End,
    allInstructions(Environment, Instructions),
    member(instruction(Start, _), Instructions),
    offsetStackFrame(Environment, Target, _),
    instructionsIncludeEnd(Instructions, End),
    currentClassLoader(Environment, CurrentLoader),
    handlerExceptionClass(Handler, ExceptionClass, CurrentLoader),
    isBootstrapLoader(BL),
    isAssignable(ExceptionClass, class('java/lang/Throwable', BL)),
    initHandlerIsLegal(Environment, Handler).

instructionsIncludeEnd(Instructions, End) :-
    member(instruction(End, _), Instructions).

instructionsIncludeEnd(Instructions, End) :-
    member(endOfCode(End), Instructions).

handlerExceptionClass(handler(_, _, _, 0),
    class('java/lang/Throwable', BL), _) :-
    isBootstrapLoader(BL).

handlerExceptionClass(handler(_, _, _, Name),
    class(Name, L), L) :-
    Name \= 0.

initHandlerIsLegal(Environment, Handler) :-
    notInitHandler(Environment, Handler).

notInitHandler(Environment, Handler) :-
    Environment = environment(_Class, Method, _, Instructions, _, _),
    isNotInit(Method).

notInitHandler(Environment, Handler) :-
    Environment = environment(_Class, Method, _, Instructions, _, _),
    isInit(Method),
    member(instruction(_, invokespecial(CP)), Instructions),
    CP = method(MethodClassName, MethodName, Descriptor),
    MethodName \= '<init>'.

initHandlerIsLegal(Environment, Handler) :-
    isInitHandler(Environment, Handler),
    sublist(isApplicableInstruction(Target), Instructions,
    HandlerInstructions),
    noAttemptToReturnNormally(HandlerInstructions).

isInitHandler(Environment, Handler) :-
    Environment = environment(_Class, Method, _, Instructions, _, _),
    isInit(Method),
    member(instruction(_, invokespecial(CP)), Instructions),
    CP = method(MethodClassName, '<init>', Descriptor).

isApplicableInstruction(HandlerStart, instruction(Offset, _)) :-
    Offset >= HandlerStart.

noAttemptToReturnNormally(Instructions) :-
    notMember(instruction(_, return), Instructions).

noAttemptToReturnNormally(Instructions) :-
    member(instruction(_, athrow), Instructions).

mergeStackMapAndCode([], CodeList, CodeList).

mergeStackMapAndCode([stackMap(Offset, Map) | RestMap], [instruction(Offset, Parse) | RestCode], [stackMap(Offset, Map), instruction(Offset, Parse) | RestMerge]) :-
    mergeStackMapAndCode(RestMap, RestCode, RestMerge).

mergeStackMapAndCode([stackMap(OffsetM, Map) | RestMap],[instruction(OffsetP, Parse) | RestCode],[instruction(OffsetP, Parse) | RestMerge]) :-
    OffsetP < OffsetM,
    mergeStackMapAndCode([stackMap(OffsetM, Map) | RestMap], RestCode, RestMerge).

methodInitialStackFrame(Class, Method, FrameSize, frame(Locals, [], Flags),ReturnType):-
    methodDescriptor(Method, Descriptor),
    parseMethodDescriptor(Descriptor, RawArgs, ReturnType),
    expandTypeList(RawArgs, Args),
    methodInitialThisType(Class, Method, ThisList),
    flags(ThisList, Flags),
    append(ThisList, Args, ThisArgs),
    expandToLength(ThisArgs, FrameSize, top, Locals).

expandTypeList([], []).

expandTypeList([Item | List], [Item | Result]) :-
    sizeOf(Item, 1),
    expandTypeList(List, Result).

expandTypeList([Item | List], [Item, top | Result]) :-
    sizeOf(Item, 2),
    expandTypeList(List, Result).

flags([uninitializedThis], [flagThisUninit]).

flags(X, []) :- X \= [uninitializedThis].

expandToLength(List, Size, _Filler, List) :-
    length(List, Size).

expandToLength(List, Size, Filler, Result) :-
    length(List, ListLength),
    ListLength < Size,
    Delta is Size - ListLength,

length(Extra, Delta),
    checklist(=(Filler), Extra),

append(List, Extra, Result).

methodInitialThisType(_Class, Method, []) :-
    methodAccessFlags(Method, AccessFlags),
    member(static, AccessFlags),
    methodName(Method, MethodName),
    MethodName \= '<init>'.

methodInitialThisType(Class, Method, [This]) :-
    methodAccessFlags(Method, AccessFlags),
    notMember(static, AccessFlags),
    instanceMethodInitialThisType(Class, Method, This).

instanceMethodInitialThisType(Class, Method, class('java/lang/Object', L)) :-
    methodName(Method, '<init>'),
    classDefiningLoader(Class, L),
    isBootstrapLoader(L),
    classClassName(Class, 'java/lang/Object').

instanceMethodInitialThisType(Class, Method, uninitializedThis) :-
    methodName(Method, '<init>'),
    classClassName(Class, ClassName),
    classDefiningLoader(Class, CurrentLoader),
    superclassChain(ClassName, CurrentLoader, Chain),
    Chain \= [].

instanceMethodInitialThisType(Class, Method, class(ClassName, L)) :-
    methodName(Method, MethodName),
    MethodName \= '<init>',
    classDefiningLoader(Class, L),
    classClassName(Class, ClassName).


mergedCodeIsTypeSafe(Environment, [stackMap(Offset, MapFrame) | MoreCode], frame(Locals, OperandStack, Flags)) :-
    frameIsAssignable(frame(Locals, OperandStack, Flags), MapFrame),
    mergedCodeIsTypeSafe(Environment, MoreCode, MapFrame).

mergedCodeIsTypeSafe(Environment, [instruction(Offset, Parse) | MoreCode], frame(Locals, OperandStack, Flags)) :-
    instructionIsTypeSafe(Parse, Environment, Offset,frame(Locals, OperandStack, Flags),NextStackFrame, ExceptionStackFrame),
    instructionSatisfiesHandlers(Environment, Offset, ExceptionStackFrame),
    mergedCodeIsTypeSafe(Environment, MoreCode, NextStackFrame).

mergedCodeIsTypeSafe(Environment, [stackMap(Offset, MapFrame) | MoreCode],afterGoto) :-
    mergedCodeIsTypeSafe(Environment, MoreCode, MapFrame).

mergedCodeIsTypeSafe(_Environment, [instruction(_, _) | _MoreCode],afterGoto) :-
    write_ln('No stack frame after unconditional branch'),
    fail.

mergedCodeIsTypeSafe(_Environment, [endOfCode(Offset)],afterGoto).

targetIsTypeSafe(Environment, StackFrame, Target) :-
    offsetStackFrame(Environment, Target, Frame),
    frameIsAssignable(StackFrame, Frame).

instructionSatisfiesHandlers(Environment, Offset, ExceptionStackFrame) :-
    exceptionHandlers(Environment, Handlers),
    sublist(isApplicableHandler(Offset), Handlers, ApplicableHandlers),
    checklist(instructionSatisfiesHandler(Environment, ExceptionStackFrame),ApplicableHandlers).

isApplicableHandler(Offset, handler(Start, End, _Target, _ClassName)) :-
    Offset >= Start,
    Offset < End.

instructionSatisfiesHandler(Environment, ExcStackFrame, Handler) :-
    Handler = handler(_, _, Target, _),
    currentClassLoader(Environment, CurrentLoader),
    handlerExceptionClass(Handler, ExceptionClass, CurrentLoader),
    /* The stack consists of just the exception. */
    ExcStackFrame = frame(Locals, _, Flags),
    TrueExcStackFrame = frame(Locals, [ ExceptionClass ], Flags),
    operandStackHasLegalLength(Environment, [ExceptionClass]),
    targetIsTypeSafe(Environment, TrueExcStackFrame, Target).

loadIsTypeSafe(Environment, Index, Type, StackFrame, NextStackFrame) :-
    StackFrame = frame(Locals, _OperandStack, _Flags),
    nth0(Index, Locals, ActualType),
    isAssignable(ActualType, Type),
    validTypeTransition(Environment, [], ActualType, StackFrame,NextStackFrame).

storeIsTypeSafe(_Environment, Index, Type,frame(Locals, OperandStack, Flags),frame(NextLocals, NextOperandStack, Flags)) :-
    popMatchingType(OperandStack, Type, NextOperandStack, ActualType),
    modifyLocalVariable(Index, ActualType, Locals, NextLocals).

modifyLocalVariable(Index, Type, Locals, NewLocals) :-
    modifyLocalVariable(0, Index, Type, Locals, NewLocals).

modifyLocalVariable(I, Index, Type,[Locals1 | LocalsRest],[Locals1 | NextLocalsRest] ) :-
    I < Index - 1,
    I1 is I + 1,
    modifyLocalVariable(I1, Index, Type, LocalsRest, NextLocalsRest).

modifyLocalVariable(I, Index, Type,[Locals1 | LocalsRest],[NextLocals1 | NextLocalsRest] ) :-
    I =:= Index - 1,
    modifyPreIndexVariable(Locals1, NextLocals1),
    modifyLocalVariable(Index, Index, Type, LocalsRest, NextLocalsRest).


modifyLocalVariable(Index, Index, Type, [_ | LocalsRest], [Type | LocalsRest]) :-
    sizeOf(Type, 1).

modifyLocalVariable(Index, Index, Type, [_, _ | LocalsRest], [Type, top | LocalsRest]) :-
    sizeOf(Type, 2).

modifyPreIndexVariable(Type, Type) :- sizeOf(Type, 1).

modifyPreIndexVariable(Type, top) :- sizeOf(Type, 2).

passesProtectedCheck(Environment, MemberClassName, MemberName,MemberDescriptor, StackFrame) :-
    thisClass(Environment, class(CurrentClassName, CurrentLoader)),
    superclassChain(CurrentClassName, CurrentLoader, Chain),
    notMember(class(MemberClassName, _), Chain).

passesProtectedCheck(Environment, MemberClassName, MemberName,MemberDescriptor, StackFrame) :-
    thisClass(Environment, class(CurrentClassName, CurrentLoader)),
    superclassChain(CurrentClassName, CurrentLoader, Chain),
    member(class(MemberClassName, _), Chain),
    classesInOtherPkgWithProtectedMember(class(CurrentClassName, CurrentLoader),MemberName, MemberDescriptor, MemberClassName, Chain, []).

passesProtectedCheck(Environment, MemberClassName, MemberName,MemberDescriptor,frame(_Locals, [Target | Rest], _Flags)) :-
    thisClass(Environment, class(CurrentClassName, CurrentLoader)),
    superclassChain(CurrentClassName, CurrentLoader, Chain),
    member(class(MemberClassName, _), Chain),
    classesInOtherPkgWithProtectedMember(class(CurrentClassName, CurrentLoader),MemberName, MemberDescriptor, MemberClassName, Chain, List),
    List \= [],
    loadedClass(MemberClassName, CurrentLoader, ReferencedClass),
    isNotProtected(ReferencedClass, MemberName, MemberDescriptor).

passesProtectedCheck(Environment, MemberClassName, MemberName,MemberDescriptor,frame(_Locals, [Target | Rest], _Flags)) :-
    thisClass(Environment, class(CurrentClassName, CurrentLoader)),
    superclassChain(CurrentClassName, CurrentLoader, Chain),
    member(class(MemberClassName, _), Chain),
    classesInOtherPkgWithProtectedMember(class(CurrentClassName, CurrentLoader),MemberName, MemberDescriptor, MemberClassName, Chain, List),
    List \= [],
    loadedClass(MemberClassName, CurrentLoader, ReferencedClass),
    isProtected(ReferencedClass, MemberName, MemberDescriptor),
    isAssignable(Target, class(CurrentClassName, CurrentLoader)).

classesInOtherPkgWithProtectedMember(_, _, _, _, [], []).

classesInOtherPkgWithProtectedMember(Class, MemberName,
    MemberDescriptor, MemberClassName,
    [class(MemberClassName, L) | Tail],
    [class(MemberClassName, L) | T]) :-
    differentRuntimePackage(Class, class(MemberClassName, L)),
    loadedClass(MemberClassName, L, Super),
    isProtected(Super, MemberName, MemberDescriptor),
    classesInOtherPkgWithProtectedMember(Class, MemberName, MemberDescriptor, MemberClassName, Tail, T).

classesInOtherPkgWithProtectedMember(Class, MemberName,MemberDescriptor, MemberClassName,[class(MemberClassName, L) | Tail],T) :-
    differentRuntimePackage(Class, class(MemberClassName, L)),
    loadedClass(MemberClassName, L, Super),
    isNotProtected(Super, MemberName, MemberDescriptor),
    classesInOtherPkgWithProtectedMember(Class, MemberName, MemberDescriptor, MemberClassName, Tail, T).

classesInOtherPkgWithProtectedMember(Class, MemberName,
                                    MemberDescriptor, MemberClassName,
                                    [class(MemberClassName, L) | Tail],
                                    T) :-
    sameRuntimePackage(Class, class(MemberClassName, L)),
    classesInOtherPkgWithProtectedMember(Class, MemberName, MemberDescriptor, MemberClassName, Tail, T).

sameRuntimePackage(Class1, Class2) :-
    classDefiningLoader(Class1, L),
    classDefiningLoader(Class2, L),
    samePackageName(Class1, Class2).

differentRuntimePackage(Class1, Class2) :-
    classDefiningLoader(Class1, L1),
    classDefiningLoader(Class2, L2),
    L1 \= L2.

differentRuntimePackage(Class1, Class2) :-
    differentPackageName(Class1, Class2).


exceptionStackFrame(StackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, _OperandStack, Flags),
    ExceptionStackFrame = frame(Locals, [], Flags).

%instruction typesafeness starts here

instructionIsTypeSafe(Instruction, Environment, Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    instructionHasEquivalentTypeRule(Instruction, IsomorphicInstruction),
    instructionIsTypeSafe(IsomorphicInstruction, Environment, Offset,StackFrame, NextStackFrame,ExceptionStackFrame).

instructionIsTypeSafe(aaload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    nth1OperandStackIs(2, StackFrame, ArrayType),
    arrayComponentType(ArrayType, ComponentType),
    isBootstrapLoader(BL),
    validTypeTransition(Environment,[int, arrayOf(class('java/lang/Object', BL))],ComponentType, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

arrayComponentType(arrayOf(X), X).
arrayComponentType(null, null).

instructionIsTypeSafe(aastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    isBootstrapLoader(BL),
    canPop(StackFrame,[class('java/lang/Object', BL),int,arrayOf(class('java/lang/Object', BL))],NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(aconst_null, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [], null, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(aload(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadIsTypeSafe(Environment, Index, reference, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(aload_0,aload(0)).

instructionHasEquivalentTypeRule(aload_1,aload(1)).

instructionHasEquivalentTypeRule(aload_2,aload(2)).

instructionHasEquivalentTypeRule(aload_3,aload(3)).

instructionIsTypeSafe(anewarray(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    (CP = class(_, _) ; CP = arrayOf(_)),
    validTypeTransition(Environment, [int], arrayOf(CP),StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(areturn, Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    thisMethodReturnType(Environment, ReturnType),
    isAssignable(ReturnType, reference),
    canPop(StackFrame, [ReturnType], _PoppedStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(arraylength, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    nth1OperandStackIs(1, StackFrame, ArrayType),
    arrayComponentType(ArrayType, _),
    validTypeTransition(Environment, [top], int, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(astore(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    storeIsTypeSafe(Environment, Index, reference, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(astore_0,astore(0)).

instructionHasEquivalentTypeRule(astore_1,astore(1)).

instructionHasEquivalentTypeRule(astore_2,astore(2)).

instructionHasEquivalentTypeRule(astore_3,astore(3)).

instructionIsTypeSafe(athrow, _Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    isBootstrapLoader(BL),
    canPop(StackFrame, [class('java/lang/Throwable', BL)], _PoppedStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(baload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    nth1OperandStackIs(2, StackFrame, ArrayType),
    isSmallArray(ArrayType),
    validTypeTransition(Environment, [int, top], int,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

isSmallArray(arrayOf(byte)).

isSmallArray(arrayOf(boolean)).

isSmallArray(null).

instructionIsTypeSafe(bastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    nth1OperandStackIs(3, StackFrame, ArrayType),
    isSmallArray(ArrayType),
    canPop(StackFrame, [int, int, top], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(bipush(Value), sipush(Value)).

instructionIsTypeSafe(caload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, arrayOf(char)], int,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(castore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [int, int, arrayOf(char)], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(checkcast(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    (CP = class(_, _) ; CP = arrayOf(_)),
    isBootstrapLoader(BL),
    validTypeTransition(Environment, [class('java/lang/Object', BL)], CP,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(d2f, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [double], float,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(d2i, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [double], int,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(d2l, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [double], long,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(dadd, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [double, double], double,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(daload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, arrayOf(double)], double,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(dastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [double, int, arrayOf(double)], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(dcmpg, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [double, double], int,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(dcmpl, dcmpg).

instructionIsTypeSafe(dconst_0, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [], double, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(dconst_1, dconst_0).

instructionHasEquivalentTypeRule(ddiv, dadd).

instructionIsTypeSafe(dload(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadIsTypeSafe(Environment, Index, double, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(dload_0,dload(0)).

instructionHasEquivalentTypeRule(dload_1,dload(1)).

instructionHasEquivalentTypeRule(dload_2,dload(2)).

instructionHasEquivalentTypeRule(dload_3,dload(3)).

instructionHasEquivalentTypeRule(dmul, dadd).

instructionIsTypeSafe(dneg, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [double], double, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(drem, dadd).


instructionIsTypeSafe(dreturn, Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    thisMethodReturnType(Environment, double),
    canPop(StackFrame, [double], _PoppedStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(dstore(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    storeIsTypeSafe(Environment, Index, double, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(dstore_0,dstore(0)).

instructionHasEquivalentTypeRule(dstore_1,dstore(1)).

instructionHasEquivalentTypeRule(dstore_2,dstore(2)).

instructionHasEquivalentTypeRule(dstore_3,dstore(3)).

instructionHasEquivalentTypeRule(dsub, dadd).

instructionIsTypeSafe(dup, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    popCategory1(InputOperandStack, Type, _),
    canSafelyPush(Environment, InputOperandStack, Type, OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(dup_x1, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    popCategory1(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Rest),
    canSafelyPushList(Environment, Rest, [Type1, Type2, Type1],
    OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(dup_x2, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    dup_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

dup_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup_x2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup_x2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).


dup_x2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory1(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Stack2),
    popCategory1(Stack2, Type3, Rest),
    canSafelyPushList(Environment, Rest, [Type1, Type3, Type2, Type1],OutputOperandStack).

dup_x2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory1(InputOperandStack, Type1, Stack1),
    popCategory2(Stack1, Type2, Rest),
    canSafelyPushList(Environment, Rest, [Type1, Type2, Type1],OutputOperandStack).

instructionIsTypeSafe(dup2, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    dup2FormIsTypeSafe(Environment,InputOperandStack, OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

dup2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2Form1IsTypeSafe(Environment,InputOperandStack, OutputOperandStack).

dup2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2Form2IsTypeSafe(Environment,InputOperandStack, OutputOperandStack).


dup2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack):-
    popCategory1(InputOperandStack, Type1, TempStack),
    popCategory1(TempStack, Type2, _),
    canSafelyPushList(Environment, InputOperandStack, [Type2, Type1],
    OutputOperandStack).

dup2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack):-
    popCategory2(InputOperandStack, Type, _),
    canSafelyPush(Environment, InputOperandStack, Type, OutputOperandStack).

instructionIsTypeSafe(dup2_x1, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    dup2_x1FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

dup2_x1FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2_x1Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup2_x1FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2_x1Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup2_x1Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory1(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Stack2),
    popCategory1(Stack2, Type3, Rest),
    canSafelyPushList(Environment, Rest, [Type2, Type1, Type3, Type2, Type1],OutputOperandStack).

dup2_x1Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory2(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Rest),
    canSafelyPushList(Environment, Rest, [Type1, Type2, Type1],OutputOperandStack).

instructionIsTypeSafe(dup2_x2, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2_x2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2_x2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2_x2Form3IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    dup2_x2Form4IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).

dup2_x2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory1(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Stack2),
    popCategory1(Stack2, Type3, Stack3),
    popCategory1(Stack3, Type4, Rest),
    canSafelyPushList(Environment, Rest,[Type2, Type1, Type4, Type3, Type2, Type1],OutputOperandStack).

dup2_x2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory2(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Stack2),
    popCategory1(Stack2, Type3, Rest),
    canSafelyPushList(Environment, Rest,[Type1, Type3, Type2, Type1],OutputOperandStack).

dup2_x2Form3IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory1(InputOperandStack, Type1, Stack1),
    popCategory1(Stack1, Type2, Stack2),
    popCategory2(Stack2, Type3, Rest),
    canSafelyPushList(Environment, Rest,[Type2, Type1, Type3, Type2, Type1],OutputOperandStack).

dup2_x2Form4IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
    popCategory2(InputOperandStack, Type1, Stack1),
    popCategory2(Stack1, Type2, Rest),
    canSafelyPushList(Environment, Rest, [Type1, Type2, Type1],OutputOperandStack).

instructionIsTypeSafe(f2d, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [float], double,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(f2i, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [float], int,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(f2l, Environment, _Offset, StackFrame, NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [float], long,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(fadd, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [float, float], float,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(faload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, arrayOf(float)], float,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(fastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [float, int, arrayOf(float)], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(fcmpg, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [float, float], int,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(fcmpl, fcmpg).

instructionIsTypeSafe(fconst_0, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [], float, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(fconst_1, fconst_0).
instructionHasEquivalentTypeRule(fconst_2, fconst_0).

instructionHasEquivalentTypeRule(fdiv, fadd).

instructionIsTypeSafe(fload(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadIsTypeSafe(Environment, Index, float, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(fload_0,fload(0)).

instructionHasEquivalentTypeRule(fload_1,fload(1)).

instructionHasEquivalentTypeRule(fload_2,fload(2)).

instructionHasEquivalentTypeRule(fload_3,fload(3)).

instructionHasEquivalentTypeRule(fmul, fadd).

instructionIsTypeSafe(fneg, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [float], float,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(frem, fadd).

instructionIsTypeSafe(freturn, Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    thisMethodReturnType(Environment, float),
    canPop(StackFrame, [float], _PoppedStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(fstore(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    storeIsTypeSafe(Environment, Index, float, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(fstore_0,fstore(0)).

instructionHasEquivalentTypeRule(fstore_1,fstore(1)).

instructionHasEquivalentTypeRule(fstore_2,fstore(2)).

instructionHasEquivalentTypeRule(fstore_3,fstore(3)).

instructionHasEquivalentTypeRule(fsub, fadd).


instructionIsTypeSafe(getfield(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = field(FieldClassName, FieldName, FieldDescriptor),
    parseFieldDescriptor(FieldDescriptor, FieldType),
    passesProtectedCheck(Environment, FieldClassName, FieldName,
    FieldDescriptor, StackFrame),
    currentClassLoader(Environment, CurrentLoader),
    validTypeTransition(Environment,
    [class(FieldClassName, CurrentLoader)], FieldType,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(getstatic(CP), Environment, _Offset, StackFrame,
    NextStackFrame, ExceptionStackFrame) :-
    CP = field(_FieldClassName, _FieldName, FieldDescriptor),
    parseFieldDescriptor(FieldDescriptor, FieldType),
    validTypeTransition(Environment, [], FieldType,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(goto(Target), Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    targetIsTypeSafe(Environment, StackFrame, Target),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(goto_w(Target), goto(Target)).

instructionHasEquivalentTypeRule(i2b, ineg).

instructionHasEquivalentTypeRule(i2c, ineg).

instructionIsTypeSafe(i2d, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int], double,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(i2f, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int], float,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(i2s, ineg).

instructionIsTypeSafe(iadd, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, int], int,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(iaload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, arrayOf(int)], int,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(iand, iadd).

instructionIsTypeSafe(iastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [int, int, arrayOf(int)], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(iconst_m1, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [], int, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(iconst_0,iconst_m1).

instructionHasEquivalentTypeRule(iconst_1,iconst_m1).

instructionHasEquivalentTypeRule(iconst_2,iconst_m1).

instructionHasEquivalentTypeRule(iconst_3,iconst_m1).

instructionHasEquivalentTypeRule(iconst_4,iconst_m1).

instructionHasEquivalentTypeRule(iconst_5,iconst_m1).

instructionHasEquivalentTypeRule(idiv, iadd).

instructionIsTypeSafe(if_acmpeq(Target), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [reference, reference], NextStackFrame),
    targetIsTypeSafe(Environment, NextStackFrame, Target),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(if_acmpne(Target), if_acmpeq(Target)).

instructionIsTypeSafe(if_icmpeq(Target), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [int, int], NextStackFrame),
    targetIsTypeSafe(Environment, NextStackFrame, Target),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(if_icmpge(Target),if_icmpeq(Target)).

instructionHasEquivalentTypeRule(if_icmpgt(Target),if_icmpeq(Target)).

instructionHasEquivalentTypeRule(if_icmple(Target),if_icmpeq(Target)).

instructionHasEquivalentTypeRule(if_icmplt(Target),if_icmpeq(Target)).

instructionHasEquivalentTypeRule(if_icmpne(Target),if_icmpeq(Target)).

instructionIsTypeSafe(ifeq(Target), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [int], NextStackFrame),
    targetIsTypeSafe(Environment, NextStackFrame, Target),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(ifge(Target),ifeq(Target)).

instructionHasEquivalentTypeRule(ifgt(Target),ifeq(Target)).

instructionHasEquivalentTypeRule(ifle(Target),ifeq(Target)).

instructionHasEquivalentTypeRule(iflt(Target),ifeq(Target)).

instructionHasEquivalentTypeRule(ifne(Target),ifeq(Target)).

instructionIsTypeSafe(ifnonnull(Target), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [reference], NextStackFrame),
    targetIsTypeSafe(Environment, NextStackFrame, Target),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(ifnull(Target), ifnonnull(Target)).

instructionIsTypeSafe(iinc(Index, _Value), _Environment, _Offset,StackFrame, StackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, _OperandStack, _Flags),
    nth0(Index, Locals, int),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(iload(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadIsTypeSafe(Environment, Index, int, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(iload_0,iload(0)).
instructionHasEquivalentTypeRule(iload_1,iload(1)).
instructionHasEquivalentTypeRule(iload_2,iload(2)).
instructionHasEquivalentTypeRule(iload_3,iload(3)).


instructionHasEquivalentTypeRule(imul, iadd).

instructionIsTypeSafe(ineg, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int], int, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(instanceof(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    (CP = class(_, _) ; CP = arrayOf(_)),
    isBootstrapLoader(BL),
    validTypeTransition(Environment, [class('java/lang/Object', BL)], int,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(invokedynamic(CP,0,0), Environment, _Offset,StackFrame, NextStackFrame, ExceptionStackFrame) :-
    CP = dmethod(CallSiteName, Descriptor),
    CallSiteName \= '<init>',
    CallSiteName \= ' <clinit> ',
    parseMethodDescriptor(Descriptor, OperandArgList, ReturnType),
    reverse(OperandArgList, StackArgList),
    validTypeTransition(Environment, StackArgList, ReturnType,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(invokeinterface(CP, Count, 0), Environment, _Offset,StackFrame, NextStackFrame, ExceptionStackFrame) :-
    CP = imethod(MethodIntfName, MethodName, Descriptor),
    MethodName \= '<init>',
    MethodName \= ' <clinit> ',
    parseMethodDescriptor(Descriptor, OperandArgList, ReturnType),
    currentClassLoader(Environment, CurrentLoader),
    reverse([class(MethodIntfName, CurrentLoader) | OperandArgList],
    StackArgList),
    canPop(StackFrame, StackArgList, TempFrame),
    validTypeTransition(Environment, [], ReturnType,
    TempFrame, NextStackFrame),
    countIsValid(Count, StackFrame, TempFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

countIsValid(Count, InputFrame, OutputFrame) :-
    InputFrame = frame(_Locals1, OperandStack1, _Flags1),
    OutputFrame = frame(_Locals2, OperandStack2, _Flags2),
    length(OperandStack1, Length1),
    length(OperandStack2, Length2),
    Count =:= Length1 - Length2.


instructionIsTypeSafe(invokespecial(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = method(MethodClassName, MethodName, Descriptor),
    MethodClassName = arrayOf(_),
    write_ln('unimplemented'),
    fail.

instructionIsTypeSafe(invokespecial(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = method(MethodClassName, MethodName, Descriptor),
    MethodName \= '<init>',
    MethodName \= ' <clinit> ',
    parseMethodDescriptor(Descriptor, OperandArgList, ReturnType),
    thisClass(Environment, class(CurrentClassName, CurrentLoader)),
    isAssignable(class(CurrentClassName, CurrentLoader),
    class(MethodClassName, CurrentLoader)),
    reverse([class(CurrentClassName, CurrentLoader) | OperandArgList],
    StackArgList),
    validTypeTransition(Environment, StackArgList, ReturnType,
    StackFrame, NextStackFrame),
    reverse([class(MethodClassName, CurrentLoader) | OperandArgList],
    StackArgList2),
    validTypeTransition(Environment, StackArgList2, ReturnType,
    StackFrame, _ResultStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(invokespecial(CP), Environment, _Offset, StackFrame,
    NextStackFrame, ExceptionStackFrame) :-
    CP = method(MethodClassName, '<init>', Descriptor),
    parseMethodDescriptor(Descriptor, OperandArgList, void),
    reverse(OperandArgList, StackArgList),
    canPop(StackFrame, StackArgList, TempFrame),
    TempFrame = frame(Locals, [uninitializedThis | OperandStack], Flags),
    currentClassLoader(Environment, CurrentLoader),
    rewrittenUninitializedType(uninitializedThis, Environment,
    class(MethodClassName, CurrentLoader), This),
    rewrittenInitializationFlags(uninitializedThis, Flags, NextFlags),
    substitute(uninitializedThis, This, OperandStack, NextOperandStack),
    substitute(uninitializedThis, This, Locals, NextLocals),
    NextStackFrame = frame(NextLocals, NextOperandStack, NextFlags),
    ExceptionStackFrame = frame(Locals, [], Flags).

instructionIsTypeSafe(invokespecial(CP), Environment, _Offset, StackFrame,
    NextStackFrame, ExceptionStackFrame) :-
    CP = method(MethodClassName, '<init>', Descriptor),
    parseMethodDescriptor(Descriptor, OperandArgList, void),
    reverse(OperandArgList, StackArgList),
    canPop(StackFrame, StackArgList, TempFrame),
    TempFrame = frame(Locals, [uninitialized(Address) | OperandStack], Flags),
    currentClassLoader(Environment, CurrentLoader),
    rewrittenUninitializedType(uninitialized(Address), Environment,
    class(MethodClassName, CurrentLoader), This),
    rewrittenInitializationFlags(uninitialized(Address), Flags, NextFlags),
    substitute(uninitialized(Address), This, OperandStack, NextOperandStack),
    substitute(uninitialized(Address), This, Locals, NextLocals),
    NextStackFrame = frame(NextLocals, NextOperandStack, NextFlags),
    ExceptionStackFrame = frame(Locals, [], Flags),
    passesProtectedCheck(Environment, MethodClassName, '<init>',
    Descriptor, NextStackFrame).

rewrittenUninitializedType(uninitializedThis, Environment,MethodClass, This) :-
    MethodClass = class(MethodClassName, CurrentLoader),
    thisClass(Environment, This).

rewrittenUninitializedType(uninitializedThis, Environment,MethodClass, This) :-
    MethodClass = class(MethodClassName, CurrentLoader),
    thisClass(Environment, class(ThisClassName, ThisLoader)),
    superclassChain(ThisClassName, ThisLoader, [MethodClass | Rest]),
    This = class(ThisClassName, ThisLoader).

rewrittenUninitializedType(uninitialized(Address), Environment,MethodClass, This) :-
    allInstructions(Environment, Instructions),
    member(instruction(Address, new(This)), Instructions).

rewrittenInitializationFlags(uninitializedThis, _Flags, []).

rewrittenInitializationFlags(uninitialized(_), Flags, Flags).

substitute(_Old, _New, [], []).

substitute(Old, New, [Old | FromRest], [New | ToRest]) :-

substitute(Old, New, FromRest, ToRest).

substitute(Old, New, [From1 | FromRest], [From1 | ToRest]) :-
    From1 \= Old,
    substitute(Old, New, FromRest, ToRest).

instructionIsTypeSafe(invokestatic(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = method(_MethodClassName, MethodName, Descriptor),
    MethodName \= '<init>',
    MethodName \= ' <clinit> ',
    parseMethodDescriptor(Descriptor, OperandArgList, ReturnType),
    reverse(OperandArgList, StackArgList),
    validTypeTransition(Environment, StackArgList, ReturnType,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(invokevirtual(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = method(MethodClassName, MethodName, Descriptor),
    MethodName \= '<init>',
    MethodName \= ' <clinit> ',
    parseMethodDescriptor(Descriptor, OperandArgList, ReturnType),
    reverse(OperandArgList, ArgList),
    currentClassLoader(Environment, CurrentLoader),
    reverse([class(MethodClassName, CurrentLoader) | OperandArgList],StackArgList),
    validTypeTransition(Environment, StackArgList, ReturnType,StackFrame, NextStackFrame),
    canPop(StackFrame, ArgList, PoppedFrame),
    passesProtectedCheck(Environment, MethodClassName, MethodName,Descriptor, PoppedFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

% todo duplication
instructionIsTypeSafe(invokevirtual(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = method(MethodClassName, MethodName, Descriptor),
    MethodClassName = arrayOf(_),
    MethodName \= '<init>',
    MethodName \= ' <clinit> ',
    parseMethodDescriptor(Descriptor, OperandArgList, ReturnType),
    reverse(OperandArgList, ArgList),
    currentClassLoader(Environment, CurrentLoader),
    reverse([MethodClassName | OperandArgList],StackArgList),
    validTypeTransition(Environment, StackArgList, ReturnType,StackFrame, NextStackFrame),
    canPop(StackFrame, ArgList, PoppedFrame),
    passesProtectedCheck(Environment, MethodClassName, MethodName,Descriptor, PoppedFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(ior, iadd).
instructionHasEquivalentTypeRule(irem, iadd).

instructionIsTypeSafe(ireturn, Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    thisMethodReturnType(Environment, int),
    canPop(StackFrame, [int], _PoppedStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(ishl, iadd).
instructionHasEquivalentTypeRule(ishr, iadd).
instructionHasEquivalentTypeRule(iushr, iadd).

instructionIsTypeSafe(istore(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    storeIsTypeSafe(Environment, Index, int, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(istore_0,istore(0)).
instructionHasEquivalentTypeRule(istore_1,istore(1)).
instructionHasEquivalentTypeRule(istore_2,istore(2)).
instructionHasEquivalentTypeRule(istore_3,istore(3)).


instructionHasEquivalentTypeRule(isub, iadd).

instructionHasEquivalentTypeRule(ixor, iadd).

instructionIsTypeSafe(l2d, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [long], double,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(l2f, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [long], float,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(l2i, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [long], int,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(ladd, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [long, long], long,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(laload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, arrayOf(long)], long,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(land, ladd).

instructionIsTypeSafe(lastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [long, int, arrayOf(long)], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(lcmp, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [long, long], int,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(lconst_0, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [], long, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(lconst_1, lconst_0).

instructionIsTypeSafe(ldc(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadableConstant(CP, Type),
    Type \= long,
    Type \= double,
    validTypeTransition(Environment, [], Type, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

loadableConstant(CP, Type) :-
    member([CP, Type], [[int(_),int],[float(_), float],[long(_),long],[double(_), double]]).

loadableConstant(CP, Type) :-
    isBootstrapLoader(BL),member([CP, Type], [[class(_),class('java/lang/Class', BL)],[string(_),class('java/lang/String', BL)],[methodHandle(_,_), class('java/lang/invoke/MethodHandle', BL)],[methodType(_,_),class('java/lang/invoke/MethodType', BL)]]).

loadableConstant(CP, Type) :-
    CP = dconstant(_, FieldDescriptor),
    parseFieldDescriptor(FieldDescriptor, Type).

instructionHasEquivalentTypeRule(ldc_w(CP), ldc(CP)).

instructionIsTypeSafe(ldc2_w(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadableConstant(CP, Type),
    (Type = long ; Type = double),
    validTypeTransition(Environment, [], Type, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(ldiv, ladd).

instructionIsTypeSafe(lload(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    loadIsTypeSafe(Environment, Index, long, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(lload_0,lload(0)).
instructionHasEquivalentTypeRule(lload_1,lload(1)).
instructionHasEquivalentTypeRule(lload_2,lload(2)).
instructionHasEquivalentTypeRule(lload_3,lload(3)).


instructionHasEquivalentTypeRule(lmul, ladd).


instructionIsTypeSafe(lneg, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [long], long,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(lookupswitch(Targets, Keys), Environment, _, StackFrame,afterGoto, ExceptionStackFrame) :-
    sort(Keys, Keys),
    canPop(StackFrame, [int], BranchStackFrame),
    checklist(targetIsTypeSafe(Environment, BranchStackFrame), Targets),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(lor, ladd).
instructionHasEquivalentTypeRule(lrem, ladd).

instructionIsTypeSafe(lreturn, Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    thisMethodReturnType(Environment, long),
    canPop(StackFrame, [long], _PoppedStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(lshl, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, long], long,StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(lshr, lshl).

instructionHasEquivalentTypeRule(lushr, lshl).

instructionIsTypeSafe(lstore(Index), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    storeIsTypeSafe(Environment, Index, long, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(lstore_0,lstore(0)).
instructionHasEquivalentTypeRule(lstore_1,lstore(1)).
instructionHasEquivalentTypeRule(lstore_2,lstore(2)).
instructionHasEquivalentTypeRule(lstore_3,lstore(3)).

instructionHasEquivalentTypeRule(lsub, ladd).

instructionHasEquivalentTypeRule(lxor, ladd).

instructionIsTypeSafe(monitorenter, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [reference], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(monitorexit, monitorenter).

instructionIsTypeSafe(multianewarray(CP, Dim), Environment, _Offset,StackFrame, NextStackFrame, ExceptionStackFrame) :-
    CP = arrayOf(_),
    classDimension(CP, Dimension),
    Dimension >= Dim,
    Dim > 0,
    /* Make a list of Dim ints */
    findall(int, between(1, Dim, _), IntList),
    validTypeTransition(Environment, IntList, CP,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

classDimension(arrayOf(X), Dimension) :-
    classDimension(X, Dimension1),
    Dimension is Dimension1 + 1.
classDimension(_, Dimension) :-
    Dimension = 0.


instructionIsTypeSafe(new(CP), Environment, Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, OperandStack, Flags),
    CP = class(_, _),
    NewItem = uninitialized(Offset),
    notMember(NewItem, OperandStack),
    substitute(NewItem, top, Locals, NewLocals),
    validTypeTransition(Environment, [], NewItem,
    frame(NewLocals, OperandStack, Flags),NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(newarray(TypeCode), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    primitiveArrayInfo(TypeCode, _TypeChar, ElementType, _VerifierType),
    validTypeTransition(Environment, [int], arrayOf(ElementType),
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

primitiveArrayInfo(4,0'Z,boolean,int).
primitiveArrayInfo(5,0'C,char,int).
primitiveArrayInfo(6,0'F,float,float).
primitiveArrayInfo(7,0'D,double,double).
primitiveArrayInfo(8,0'B,byte,int).
primitiveArrayInfo(9,0'S,short,int).
primitiveArrayInfo(10,0'I,int,int).
primitiveArrayInfo(11,0'J,long,long).



instructionIsTypeSafe(nop, _Environment, _Offset, StackFrame,StackFrame, ExceptionStackFrame) :-
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(pop, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, [Type | Rest], Flags),
    popCategory1([Type | Rest], Type, Rest),
    NextStackFrame = frame(Locals, Rest, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(pop2, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(Locals, InputOperandStack, Flags),
    pop2SomeFormIsTypeSafe(InputOperandStack, OutputOperandStack),
    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

pop2SomeFormIsTypeSafe(InputOperandStack, OutputOperandStack) :-
    pop2Form1IsTypeSafe(InputOperandStack, OutputOperandStack).

pop2SomeFormIsTypeSafe(InputOperandStack, OutputOperandStack) :-
    pop2Form2IsTypeSafe(InputOperandStack, OutputOperandStack).


pop2Form1IsTypeSafe([Type1, Type2 | Rest], Rest) :-
    popCategory1([Type1 | Rest], Type1, Rest),
    popCategory1([Type2 | Rest], Type2, Rest).

pop2Form2IsTypeSafe([top, Type | Rest], Rest) :-
    popCategory2([top, Type | Rest], Type, Rest).

instructionIsTypeSafe(putfield(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = field(FieldClassName, FieldName, FieldDescriptor),
    parseFieldDescriptor(FieldDescriptor, FieldType),
    canPop(StackFrame, [FieldType], PoppedFrame),
    passesProtectedCheck(Environment, FieldClassName, FieldName,FieldDescriptor, PoppedFrame),
    currentClassLoader(Environment, CurrentLoader),
    canPop(StackFrame, [FieldType, class(FieldClassName, CurrentLoader)],NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(putfield(CP), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = field(FieldClassName, _FieldName, FieldDescriptor),
    parseFieldDescriptor(FieldDescriptor, FieldType),
    Environment = environment(CurrentClass, CurrentMethod, _, _, _, _),
    CurrentClass = class(FieldClassName, _),
    isInit(CurrentMethod),
    canPop(StackFrame, [FieldType, uninitializedThis], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(putstatic(CP), _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    CP = field(_FieldClassName, _FieldName, FieldDescriptor),
    parseFieldDescriptor(FieldDescriptor, FieldType),
    canPop(StackFrame, [FieldType], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).


instructionIsTypeSafe(return, Environment, _Offset, StackFrame,afterGoto, ExceptionStackFrame) :-
    thisMethodReturnType(Environment, void),
    StackFrame = frame(_Locals, _OperandStack, Flags),
    notMember(flagThisUninit, Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(saload, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [int, arrayOf(short)], int,
    StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(sastore, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    canPop(StackFrame, [int, int, arrayOf(short)], NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(sipush(_Value), Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    validTypeTransition(Environment, [], int, StackFrame, NextStackFrame),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(swap, _Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
    StackFrame = frame(_Locals, [Type1, Type2 | Rest], _Flags),
    popCategory1([Type1 | Rest], Type1, Rest),
    popCategory1([Type2 | Rest], Type2, Rest),
    NextStackFrame = frame(_Locals, [Type2, Type1 | Rest], _Flags),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionIsTypeSafe(tableswitch(Targets, Keys), Environment, _Offset,StackFrame, afterGoto, ExceptionStackFrame) :-
    sort(Keys, Keys),
    canPop(StackFrame, [int], BranchStackFrame),
    checklist(targetIsTypeSafe(Environment, BranchStackFrame), Targets),
    exceptionStackFrame(StackFrame, ExceptionStackFrame).

instructionHasEquivalentTypeRule(wide(WidenedInstruction),WidenedInstruction).

classIsInterface(todo).

differentPackageName(class(Name1,_), class(Name2,_)) :- differentPackageNameImpl(Name1,Name2).
differentPackageNameImpl(Name1,Name2) :-
    split_string(Name1,"/","/",Split1),
    split_string(Name2,"/","/",Split2),
    append(Packages1,[ObjectName1],Split1),
    append(Packages2,[ObjectName2],Split2),
    Packages1 \= Packages2.
samePackageName(class(Name1,_), class(Name2,_)) :- samePackageNameImpl(Name1,Name2).
samePackageNameImpl(Name1,Name2) :-
    split_string(Name1,"/","/",Split1),
    split_string(Name2,"/","/",Split2),
    append(Packages1,[ObjectName1],Split1),
    append(Packages2,[ObjectName2],Split2),
    Packages1 = Packages2.

:- discontiguous isNotStatic/2.
:- discontiguous isStatic/2.
:- discontiguous isNotPrivate/2.
:- discontiguous isPrivate/2.
:- discontiguous isNotFinal/2.
:- discontiguous isFinal/2.
:- discontiguous isNotInit/1.
:- discontiguous isInit/1.
:- discontiguous parseMethodDescriptor/3.
:- discontiguous parseFieldDescriptor/2.
