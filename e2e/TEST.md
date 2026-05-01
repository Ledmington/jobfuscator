# How to test the `classfile` library

This is an unofficial list of the largest JVM-based projects, sorted by size.
Size is intended as number of `.class` files generated in a full build (including tests and benchmarks).

Each of the following entries links to a section with its specific build instructions.

All these entries assume that [SDKMAN](https://sdkman.io/) is installed.
```bash
curl https://get.sdkman.io | bash
source ~/.sdkman/bin/sdkman-init.bash
```

After the build instructions, class files are counted with:
```
find . -type f -name "*.class" | wc -l
```

| Name                   | Link                                                | ClassFiles | Last Updated |
|------------------------|-----------------------------------------------------|------------|--------------|
| Intellij IDEA          | https://github.com/JetBrains/intellij-community.git | 144800     |  22/02/2026  |
| Kotlin                 | https://github.com/JetBrains/kotlin.git             | 141494     |  22/02/2026  |
| GraalVM                | https://github.com/oracle/graal.git                 | 53086      |  22/02/2026  |
| Apache Spark           | https://github.com/apache/spark.git                 | 36741      |  22/02/2026  |
| Gradle                 | https://github.com/gradle/gradle.git                | 35240      |  22/02/2026  |
| OpenJDK                | https://github.com/openjdk/jdk.git                  | 31120      |  22/02/2026  |
| Neo4j                  | https://github.com/neo4j/neo4j.git                  | 28675      |  20/04/2026  |
| Apache Hadoop          | https://github.com/apache/hadoop.git                | 28403      |  22/02/2026  |
| ElasticSearch          | https://github.com/elastic/elasticsearch.git        | 28397      |  23/04/2026  |
| Apache Cassandra       | https://github.com/apache/cassandra.git             | 16118      |  22/02/2026  |
| Spring Boot            | https://github.com/spring-projects/spring-boot.git  | 15600      |  23/04/2026  |
| Scala                  | https://github.com/scala/scala3.git                 | 14905      |  22/02/2026  |
| Netty                  | https://github.com/netty/netty.git                  | 12518      |  23/04/2026  |
| Groovy                 | https://github.com/apache/groovy.git                | 8794       |  22/02/2026  |
| Google Guava           | https://github.com/google/guava.git                 | 7184       |  22/02/2026  |
| jUnit                  | https://github.com/junit-team/junit-framework.git   | 6604       |  23/04/2026  |
| Sbt                    | https://github.com/sbt/sbt.git                      | 4513       |  22/02/2026  |
| LibGDX                 | https://github.com/libgdx/libgdx.git                | 4006       |  22/02/2026  |
| Clojure                | https://github.com/clojure/clojure.git              | 3864       |  22/02/2026  |
| Maven                  | https://github.com/apache/maven.git                 | 3301       |  22/02/2026  |
| Mindustry (no Android) | https://github.com/Anuken/Mindustry.git             | 3012       |  22/02/2026  |
| JMH                    | https://github.com/openjdk/jmh.git                  | 2948       |  23/04/2026  |
| jMonkeyEngine          | https://github.com/jMonkeyEngine/jmonkeyengine.git  | 2668       |  22/02/2026  |
| Apache Commons Lang    | https://github.com/apache/commons-lang.git          | 1266       |  22/02/2026  |
| Project Lombok         | https://github.com/projectlombok/lombok.git         | 1072       |  20/04/2026  |
| Fernflower             | https://github.com/JetBrains/fernflower.git         | 696        |  20/04/2026  |
| JITWatch               | https://github.com/AdoptOpenJDK/jitwatch.git        | 566        |  20/04/2026  |

## Neo4j
```
sdk install java 21.0.10-tem
sdk install maven
git clone https://github.com/neo4j/neo4j.git
cd neo4j
mvn install -DskipTests -T1C
```

## JMH
```
sdk install java 25.0.2-tem
sdk install maven
git clone https://github.com/openjdk/jmh.git
cd jmh
mvn install -DskipTests -T1C
```

## jUnit
```
sdk install java 25.0.2-tem
sdk install gradle
git clone https://github.com/junit-team/junit-framework.git
cd junit-framework
./gradlew build
```

## Fernflower
```
sdk install java 25.0.2-tem
sdk install gradle
git clone https://github.com/JetBrains/fernflower.git
cd fernflower
./gradlew build
```

## JITWatch
```
sdk install java 25.0.2-tem
sdk install maven
git clone https://github.com/AdoptOpenJDK/jitwatch.git
cd jitwatch
mvn package -DskipTests -T1C
```

## Netty
```
sdk install java 25.0.2-tem
sdk install maven
wget https://github.com/netty/netty/archive/refs/tags/netty-4.2.12.Final.tar.gz
tar xzf netty-4.2.12.Final.tar.gz
cd netty
./mvnw install -DskipTests -T1C
```

## ElasticSearch
```
sdk install java 25.0.2-tem
sdk install gradle
wget https://github.com/elastic/elasticsearch/archive/refs/tags/v9.3.3.tar.gz
tar xzf v9.3.3.tar.gz
cd elasticsearch-9.3.3
./gradlew localDistro
```

## Spring Boot
```
sdk install java 25.0.2-tem
sdk install gradle
git clone https://github.com/spring-projects/spring-boot.git --single-branch --branch v4.0.5
cd spring-boot
./gradlew build
```

## Project Lombok
```
sdk install java 25.0.2-tem
sdk install ant
wget https://github.com/projectlombok/lombok/archive/refs/tags/v1.18.46.tar.gz
tar xzf v1.18.46.tar.gz
cd lombok-1.18.16
ant -noinput dist
```
