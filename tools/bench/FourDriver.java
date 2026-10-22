import cs.threephase.Search;
import cs.threephase.Tools;
import java.io.*;
import java.lang.reflect.Field;
import java.util.Random;

/**
 * Driver for the 4x4x4 comparison: it only calls the public API of
 * TPR-4x4x4-Solver (plus reflection to read how long its phases were).
 *
 * gen N SEED     prints N random cubes (facelet strings, URFDLB order)
 * (stdin cubes)  solves each cube and prints
 *                millis TAB l1 TAB l2 TAB l3 TAB l333 TAB solution
 *                where the solution ends with the whole-cube rotation
 *                (x, y, z letters), which is not counted in l1 .. l333.
 */
public class FourDriver {
    static Object field(Object o, String name) throws Exception {
        Field f = o.getClass().getDeclaredField(name);
        f.setAccessible(true);
        return f.get(o);
    }

    static int get(Object o, String name) throws Exception {
        return (Integer) field(o, name);
    }

    public static void main(String[] a) throws Exception {
        if (a.length > 0 && a[0].equals("gen")) {
            Random r = new Random(Long.parseLong(a[2]));
            for (int i = 0, n = Integer.parseInt(a[1]); i < n; i++) System.out.println(Tools.randomCube(r));
            return;
        }
        long t0 = System.nanoTime();
        Search.init();
        System.err.println("INIT_MS " + (System.nanoTime() - t0) / 1000000);
        Search s = new Search();
        BufferedReader in = new BufferedReader(new InputStreamReader(System.in));
        String line;
        boolean first = true;
        while ((line = in.readLine()) != null) {
            line = line.trim();
            if (line.isEmpty()) continue;
            if (first) {
                // The first solve of a Search object reads a wing buffer that
                // is not set up yet; solve the first cube once unmeasured.
                s.solution(line);
                first = false;
            }
            long t = System.nanoTime();
            String sol = s.solution(line);
            double ms = (System.nanoTime() - t) / 1e6;
            Object s333 = field(s, "search333");
            int l333 = (Integer) s333.getClass().getMethod("length").invoke(s333);
            System.out.println(String.format("%.3f\t%d\t%d\t%d\t%d\t%s", ms, get(s, "length1"), get(s, "length2"),
                    get(s, "totlen") - get(s, "length1") - get(s, "length2") - l333, l333, sol.trim()));
        }
    }
}
