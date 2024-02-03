ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "2.13.8"

lazy val root = (project in file("."))
  .settings(
    name := "k8s-interaction"
  )

libraryDependencies += "io.kubernetes" % "client-java" % "14.0.1"
libraryDependencies += "com.typesafe.akka" %% "akka-http" % "10.2.9"
libraryDependencies += "com.typesafe.akka" %% "akka-stream" % "2.6.18"
libraryDependencies += "com.typesafe.akka" %% "akka-actor-typed" % "2.6.18"
libraryDependencies += "net.debasishg" %% "redisclient" % "3.42"
libraryDependencies += "org.scala-lang" % "scala-library" % "2.13.8"
libraryDependencies += "org.json4s" %% "json4s-native" % "4.0.4"
libraryDependencies += "org.json4s" %% "json4s-core" % "4.0.4"
libraryDependencies += "org.json4s" %% "json4s-jackson" % "4.0.4"
libraryDependencies += "com.typesafe.akka" %% "akka-http-spray-json" % "10.2.9"
libraryDependencies += "io.spray" %% "spray-json" % "1.3.6"

libraryDependencies += "io.gatling.highcharts" % "gatling-charts-highcharts" % "3.7.6" % "test"
libraryDependencies += "io.gatling"            % "gatling-test-framework"    % "3.7.6" % "test"
libraryDependencies += "io.gatling" % "gatling-core" % "3.7.6" % "test"
libraryDependencies += "io.gatling" % "gatling-http" % "3.7.6" % "test"

enablePlugins(GatlingPlugin)

enablePlugins(PackPlugin)