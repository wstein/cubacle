import cs.cube555.*;
import java.io.*;
public class Cube555Driver { public static void main(String[] a) throws Exception {
 long t0=System.nanoTime(); Search.init(); long t1=System.nanoTime();
 System.err.println("INIT_MS "+(t1-t0)/1000000);
 Search s=new Search();
 BufferedReader r=new BufferedReader(new InputStreamReader(System.in)); String l; int lim=a.length>0?Integer.parseInt(a[0]):0;
 while((l=r.readLine())!=null){ long s0=System.nanoTime(); String[] o=s.solveReduction(l.trim(),lim); long s1=System.nanoTime();
  System.out.println((s1-s0)/1000000+"\t"+o[0]); }
}}
