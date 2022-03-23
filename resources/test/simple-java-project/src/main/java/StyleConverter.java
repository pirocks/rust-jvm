//
// Source code recreated from a .class file by IntelliJ IDEA
// (powered by FernFlower decompiler)
//


import java.util.Iterator;
import java.util.List;
import org.apache.logging.log4j.core.LogEvent;
import org.apache.logging.log4j.core.config.Configuration;
import org.apache.logging.log4j.core.config.plugins.Plugin;
import org.apache.logging.log4j.core.layout.PatternLayout;
import org.apache.logging.log4j.core.pattern.*;

@Plugin(
        name = "style",
        category = "Converter"
)
@ConverterKeys({"style"})
public final class StyleConverter extends LogEventPatternConverter {
    private final List<PatternFormatter> patternFormatters;
    private final String style;

    private StyleConverter(List<PatternFormatter> patternFormatters, String style) {
        super("style", "style");
        this.patternFormatters = patternFormatters;
        this.style = style;
    }

    public static StyleConverter newInstance(Configuration config, String[] options) {
        if (options.length < 1) {
            LOGGER.error("Incorrect number of options on style. Expected at least 1, received " + options.length);
            return null;
        } else if (options[0] == null) {
            LOGGER.error("No pattern supplied on style");
            return null;
        } else if (options[1] == null) {
            LOGGER.error("No style attributes provided");
            return null;
        } else {
            PatternParser parser = PatternLayout.createPatternParser(config);
            List<PatternFormatter> formatters = parser.parse(options[0]);
            String style = AnsiEscape.createSequence(options[1].split("\\s*,\\s*"));
            return new StyleConverter(formatters, style);
        }
    }

    public void format(LogEvent event, StringBuilder toAppendTo) {
        StringBuilder buf = new StringBuilder();
        Iterator i$ = this.patternFormatters.iterator();

        while(i$.hasNext()) {
            PatternFormatter formatter = (PatternFormatter)i$.next();
            formatter.format(event, buf);
        }

        if (buf.length() > 0) {
            toAppendTo.append(this.style).append(buf.toString()).append(AnsiEscape.getDefaultStyle());
        }

    }

    public boolean handlesThrowable() {
        Iterator i$ = this.patternFormatters.iterator();

        PatternFormatter formatter;
        do {
            if (!i$.hasNext()) {
                return false;
            }

            formatter = (PatternFormatter)i$.next();
        } while(!formatter.handlesThrowable());

        return true;
    }

    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append(super.toString());
        sb.append("[style=");
        sb.append(this.style);
        sb.append(", patternFormatters=");
        sb.append(this.patternFormatters);
        sb.append("]");
        return sb.toString();
    }
}
