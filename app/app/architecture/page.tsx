import type { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Autographs | Architecture",
  description:
    "End-to-end architecture for the Autographs deployment path from GitHub through OCI and Caddy.",
};

const workflowSteps = [
  {
    number: 1,
    name: "Bootstrap tenancy with Terraform",
    description:
      "The admin user runs the manual Terraform bootstrap that creates the compartment, deploy identity, policies, and state bucket.",
  },
  {
    number: 2,
    name: "Push code to GitHub",
    description:
      "The admin/developer pushes application, infrastructure, and deployment changes to the GitHub repository.",
  },
  {
    number: 3,
    name: "Validate pull requests",
    description:
      "Repository changes run the PR validation workflow before merge, including app and infrastructure checks.",
  },
  {
    number: 4,
    name: "Deploy from main",
    description:
      "A merge to main starts the deploy workflow that builds the app and prepares the runtime update.",
  },
  {
    number: 5,
    name: "Provision OCI runtime with Terraform",
    description:
      "GitHub Actions uses Terraform and the deploy identity to provision or update OCI resources, including the VCN, public subnet, runtime NSG, and VM.",
  },
  {
    number: 6,
    name: "Publish the app image",
    description:
      "The deploy workflow publishes a Git-SHA-tagged container image to GHCR for the VM to pull.",
  },
  {
    number: 7,
    name: "Serve public traffic",
    description:
      "End users reach Caddy over HTTPS through the public subnet and runtime NSG; Caddy reverse-proxies requests to the Next.js app container.",
  },
  {
    number: 8,
    name: "Read and write private data",
    description:
      "The Next.js app uses controlled server-side paths to access Autonomous Database metadata and private Object Storage images.",
  },
  {
    number: 9,
    name: "Manage TLS certificates",
    description:
      "Caddy uses Let's Encrypt to obtain and renew the public HTTPS certificate for the Autographs domain.",
  },
];

export default function ArchitecturePage() {
  return (
    <main className="architecture-shell">
      <section className="architecture-hero" aria-labelledby="architecture-title">
        <Link className="back-link" href="/">
          ← Proof of life
        </Link>
        <p className="eyebrow">System architecture</p>
        <h1 id="architecture-title">Autographs system overview</h1>
        <p className="lede">
          The goal is to serve my autograph collection images on a public
          website using only OCI Free Tier resources and GitHub CI/CD automation
          for management. This page documents the full target solution: GitHub
          owns source control, validation, image publishing, and deployment
          automation; OCI runs the Caddy-fronted Next.js app, Autonomous
          Database metadata, and private Object Storage media.
        </p>
      </section>

      <section className="diagram-card" aria-labelledby="diagram-title">
        <div className="section-heading">
          <h2 id="diagram-title">System diagram</h2>
        </div>

        <div className="architecture-diagram">
          <Image
            className="architecture-diagram-image"
            src="/architecture-diagram.svg"
            width={1280}
            height={760}
            priority
            alt="Autographs architecture diagram showing GitHub repository changes flowing through pull request CI, deploy workflow, GHCR image publishing, Terraform-managed OCI resources, an OCI runtime VM, Caddy, the internal Next.js app, and future Oracle Autonomous Database and private Object Storage services."
          />
        </div>
      </section>

      <section className="workflow-card" aria-labelledby="workflow-title">
        <div className="section-heading">
          <p className="eyebrow">Numbered flows</p>
          <h2 id="workflow-title">Workflow step definitions</h2>
        </div>
        <div className="workflow-table-wrap">
          <table className="workflow-table">
            <thead>
              <tr>
                <th scope="col">Step</th>
                <th scope="col">Workflow</th>
                <th scope="col">Definition</th>
              </tr>
            </thead>
            <tbody>
              {workflowSteps.map((step) => (
                <tr key={step.number}>
                  <td>
                    <span className="step-badge">{step.number}</span>
                  </td>
                  <td>{step.name}</td>
                  <td>{step.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  );
}
