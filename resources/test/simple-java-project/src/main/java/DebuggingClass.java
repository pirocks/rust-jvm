import com.sun.jdi.*;
import com.sun.jdi.connect.Connector;
import com.sun.jdi.connect.IllegalConnectorArgumentsException;
import com.sun.jdi.connect.spi.TransportService;
import com.sun.tools.jdi.GenericAttachingConnector;
import com.sun.tools.jdi.SocketAttachingConnector;
import com.sun.tools.jdi.SocketTransportService;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;

public class DebuggingClass {
    public static void main(String[] args) throws IOException, IllegalConnectorArgumentsException, IncompatibleThreadStateException {
        final GenericAttachingConnector connector = new SocketAttachingConnector();
        final VirtualMachine attached = connector.attach("localhost:5005", connector.defaultArguments());
        final List<ThreadReference> threads = attached.allThreads();
        for (ReferenceType aClass : attached.allClasses()) {
            System.out.println(aClass.name());
        }
        for (ThreadReference thread : threads) {
            for (StackFrame frame : thread.frames()) {
                System.out.println(frame.location().lineNumber());
            }
            System.out.println(thread.name());
        }


    }
}
