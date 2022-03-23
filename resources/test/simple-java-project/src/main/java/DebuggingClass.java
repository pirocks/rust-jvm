//import com.sun.jdi.*;
//import com.sun.jdi.connect.IllegalConnectorArgumentsException;
//import com.sun.jdi.event.*;
//import com.sun.jdi.request.BreakpointRequest;
//import com.sun.tools.jdi.GenericAttachingConnector;
//import com.sun.tools.jdi.SocketAttachingConnector;
//
//import java.io.IOException;
//import java.util.List;
//
//public class DebuggingClass {
//    public static void main(String[] args) throws IOException, IllegalConnectorArgumentsException, AbsentInformationException, IncompatibleThreadStateException {
//        final GenericAttachingConnector connector = new SocketAttachingConnector();
//        final VirtualMachine attached = connector.attach("localhost:5005", connector.defaultArguments());
//        for (ReferenceType aClass : attached.allClasses()) {
//           /* if (!aClass.name().equals("int") &&
//                    !aClass.name().equals("boolean") &&
//                    !aClass.name().equals("byte") &&
//                    !aClass.name().equals("short") &&
//                    !aClass.name().equals("float") &&
//                    !aClass.name().equals("double") &&
//                    !aClass.name().equals("long") &&
//                    !aClass.name().equals("char")) {
//                for (Method method : aClass.allMethods()) {
//                    System.out.println(method.name());
//                }
//            }*/
//            System.out.println(aClass.name());
//        }
////        attached.setDebugTraceMode(VirtualMachine.TRACE_ALL);
////        attached.suspend();
//        final ReferenceType swing = attached.classesByName("Swing").get(0);
//        final Method method = swing.methods().stream().filter(method1 -> method1.name().equals("main")).findFirst().get();
//        final Location location = method.allLineLocations().get(1);
//        final BreakpointRequest breakpointRequest = attached.eventRequestManager().createBreakpointRequest(location);
//        breakpointRequest.enable();
//
//        final List<ThreadReference> threadReferences = attached.allThreads();
//        for (ThreadReference thread : threadReferences) {
//            System.out.println(thread.name());
//            for (StackFrame frame : thread.frames()) {System.out.println(frame.location());
//            }
//            thread.resume();
//        }
//
//        attached.resume();
////        EventQueue queue = attached.eventQueue();
////        while (true) {
////            EventSet eventSet = null;
////            try {
////                eventSet = queue.remove();
////            } catch (InterruptedException e) {
////                e.printStackTrace();
////            }
////            EventIterator it = eventSet.eventIterator();
////            while (it.hasNext()) {
////                Event event = it.nextEvent();
////                if (event instanceof BreakpointEvent) {
////                    final BreakpointEvent event1 = (BreakpointEvent) event;
////                    System.out.println(event1.thread().name());
////                    final List<StackFrame> frames = event1.thread().frames();
////                    frames.forEach(stackFrame -> {
////                        final List<LocalVariable> localVariableList;
////                        try {
////                            localVariableList = stackFrame.visibleVariables();
////                            localVariableList.forEach(localVariable -> {
////                                System.out.println(localVariable.name());
////                                try {
////                                    System.out.println(localVariable.type());
////                                } catch (ClassNotLoadedException e) {
////                                    e.printStackTrace();
////                                }
////                                System.out.println(((ArrayReference) stackFrame.getValue(localVariable)).getValue(1));
////                                try {
////                                    System.out.println(stackFrame.visibleVariableByName("args"));
////                                } catch (AbsentInformationException e) {
////                                    e.printStackTrace();
////                                }
////                            });
////                        } catch (AbsentInformationException e) {
////                            e.printStackTrace();
////                        }
////
////                    });
////
////
////                    final List<ThreadReference> allThreads = attached.allThreads();
////                    allThreads.forEach(threadReference -> {
////                        final String name = threadReference.name();
////                        System.out.println(name);
////
////
////                        try {
////                            threadReference.frames().forEach(stackFrame -> {
////                                try {
////                                    stackFrame.visibleVariables().forEach(localVariable -> {
////                                        System.out.println(localVariable.name());
////                                    });
////                                } catch (AbsentInformationException e) {
////                                    e.printStackTrace();
////                                }
////                                stackFrame.getArgumentValues().forEach(value -> {
////                                    System.out.println(value.type());
////                                });
////                            });
////                        } catch (IncompatibleThreadStateException e) {
////                            e.printStackTrace();
////                        }
////                    });
////                }
////            }
////        }
//////        for (ThreadReference thread : threads) {
//////            if(thread.name().equals("Main")) {
////////                thread.interrupt();
//////                thread.interrupt();
//////                thread.suspend();
//////                System.out.println(thread.name());
//////                System.out.println(thread.status());
//////                System.out.println("Suspended:" + thread.isSuspended());
//////                try {
//////                    System.out.println(thread.isSuspended());
//////                    for (StackFrame frame : thread.frames()) {
//////                        try {
//////                            for (LocalVariable variable : frame.visibleVariables()) {
//////                                System.out.println(variable.name());
//////                                System.out.println(frame.getValue(variable));
//////                                System.out.println(variable.isArgument());
//////                                System.out.println(variable.genericSignature());
//////
//////                            }
//////                        } catch (AbsentInformationException e) {
//////                            e.printStackTrace();
//////                        }
//////                        System.out.println(frame.thisObject());
//////                        System.out.println(frame.thread());
//////                        try {
//////                            System.out.println(frame.location().lineNumber());
//////                        } catch (InternalError e) {
//////                            e.printStackTrace();
//////                        }
//////                    }
//////                    System.out.println(thread.frameCount());
//////                } catch (IncompatibleThreadStateException e) {
//////                    e.printStackTrace();
//////                }
//////
//////                System.out.println(thread.name());
//////            }
//////        }
////
////
//    }
//}

import java.lang.annotation.Annotation;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;

public class DebuggingClass{
    public static void main(String[] args) throws NoSuchFieldException, IllegalAccessException, InvocationTargetException, NoSuchMethodException {

    }

    static {
        System.out.println("foo");
    }

}
